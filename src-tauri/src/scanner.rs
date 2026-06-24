use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub url: String,
    pub follow_redirects: bool,
    pub custom_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderEntry {
    pub name: String,
    pub value: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub header: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieWarning {
    pub name: String,
    pub missing_flags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportInfo {
    pub label: String,
    pub value: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsInfo {
    pub header: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub url: String,
    pub final_url: String,
    pub status_code: u16,
    pub http_version: String,
    pub transport_security: Vec<TransportInfo>,
    pub cors: Vec<CorsInfo>,
    pub cookie_warnings: Vec<CookieWarning>,
    pub present_headers: Vec<HeaderEntry>,
    pub missing_headers: Vec<HeaderEntry>,
    pub warnings: Vec<Warning>,
    pub all_headers: HashMap<String, String>,
    pub error: Option<String>,
}

const SECURITY_HEADERS: &[(&str, &str)] = &[
    ("strict-transport-security", "HSTS enforces secure (HTTP over SSL/TLS) connections to the server."),
    ("content-security-policy", "CSP prevents cross-site scripting (XSS), clickjacking and other code injection attacks."),
    ("x-frame-options", "Protects against clickjacking attacks. (Obsolete if CSP frame-ancestors is used)"),
    ("x-content-type-options", "Prevents MIME-sniffing. Should be set to 'nosniff'."),
    ("referrer-policy", "Controls how much referrer information should be included with requests."),
    ("permissions-policy", "Allows site to control which features and APIs can be used in the browser."),
    ("cache-control", "Specifies caching policies. Should prevent sensitive data from being cached."),
    ("cross-origin-opener-policy", "Controls which documents can share a window with the document."),
    ("cross-origin-embedder-policy", "Prevents a document from loading cross-origin resources without explicit permission."),
    ("cross-origin-resource-policy", "Controls which origins are allowed to fetch the resource."),
    ("x-xss-protection", "Legacy header to enable XSS filtering in browsers. (Mostly obsolete)"),
];

const INFO_HEADERS: &[(&str, &str)] = &[
    ("server", "Reveals the server software and version."),
    ("x-powered-by", "Reveals the application framework (e.g., PHP, Express)."),
    ("x-aspnet-version", "Reveals ASP.NET version."),
    ("x-generator", "Reveals the CMS or static site generator used."),
    ("x-runtime", "Reveals execution time, often found in Ruby on Rails."),
    ("via", "Reveals proxy server details."),
    ("x-cache", "Reveals caching technology (e.g., Varnish, Squid)."),
    ("cf-ray", "Cloudflare trace ID (reveals usage of Cloudflare)."),
    ("server-timing", "Reveals backend processing times and server metrics."),
];

pub async fn scan_url(opts: ScanOptions) -> ScanResult {
    let initial_url = if opts.url.starts_with("http://") || opts.url.starts_with("https://") {
        opts.url.clone()
    } else {
        format!("http://{}", opts.url)
    };

    let alt_discovery_url = if initial_url.starts_with("https://") {
        Some((initial_url.replace("https://", "http://"), "Plain HTTP Upgrade"))
    } else if initial_url.starts_with("http://") {
        Some((initial_url.replace("http://", "https://"), "HTTPS Availability"))
    } else {
        None
    };

    let mut client_builder = Client::builder().timeout(Duration::from_secs(10));
    client_builder = if opts.follow_redirects {
        client_builder
    } else {
        client_builder.redirect(reqwest::redirect::Policy::none())
    };

    let client = match client_builder.build() {
        Ok(c) => c,
        Err(e) => return error_result(&opts.url, format!("Failed to build client: {}", e)),
    };

    let mut request_builder = client.get(&initial_url);
    for header in &opts.custom_headers {
        if let Some((k, v)) = header.split_once(':') {
            request_builder = request_builder.header(k.trim(), v.trim());
        }
    }

    let mut cert_error = None;
    let response = match request_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            let err_msg = e.to_string().to_lowercase();
            if err_msg.contains("certificate")
                || err_msg.contains("ssl")
                || err_msg.contains("tls")
                || err_msg.contains("pkix")
                || err_msg.contains("mismatch")
                || err_msg.contains("expired")
            {
                cert_error = Some(e.to_string());
                let mut insecure_builder = Client::builder().timeout(Duration::from_secs(10));
                if !opts.follow_redirects {
                    insecure_builder = insecure_builder.redirect(reqwest::redirect::Policy::none());
                }
                let insecure_client = match insecure_builder.danger_accept_invalid_certs(true).build() {
                    Ok(c) => c,
                    Err(e2) => return error_result(&opts.url, format!("Failed to build client: {}", e2)),
                };
                let mut insecure_req = insecure_client.get(&initial_url);
                for header in &opts.custom_headers {
                    if let Some((k, v)) = header.split_once(':') {
                        insecure_req = insecure_req.header(k.trim(), v.trim());
                    }
                }
                match insecure_req.send().await {
                    Ok(resp) => resp,
                    Err(e2) => return error_result(&opts.url, e2.to_string()),
                }
            } else {
                return error_result(&opts.url, e.to_string());
            }
        }
    };

    let final_url = response.url().clone().to_string();
    let status_code = response.status().as_u16();
    let http_version = format!("{:?}", response.version());
    let headers = response.headers().clone();
    let initial_is_https = initial_url.starts_with("https://");

    let mut alt_discovery_status: Option<(String, String, String)> = None;
    if let Some((disco_url, label)) = alt_discovery_url {
        if let Ok(dc) = Client::builder()
            .timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()
        {
            match dc.get(&disco_url).send().await {
                Ok(resp) => {
                    if label == "Plain HTTP Upgrade" {
                        if resp.status().is_redirection() {
                            if let Some(loc) = resp.headers().get("location") {
                                if loc.to_str().unwrap_or("").starts_with("https://") {
                                    alt_discovery_status = Some((label.into(), "SUCCESSFUL (Redirected to HTTPS)".into(), "secure".into()));
                                } else {
                                    alt_discovery_status = Some((label.into(), "INSECURE (Redirects but not to HTTPS)".into(), "warning".into()));
                                }
                            } else {
                                alt_discovery_status = Some((label.into(), "INSECURE (Redirects but missing location header)".into(), "warning".into()));
                            }
                        } else if resp.status().is_success() {
                            alt_discovery_status = Some((label.into(), "INSECURE (HTTP accessible, does not redirect)".into(), "insecure".into()));
                        } else {
                            alt_discovery_status = Some((label.into(), format!("UNKNOWN (HTTP status {})", resp.status().as_u16()), "info".into()));
                        }
                    } else if resp.status().is_success() || resp.status().is_redirection() {
                        alt_discovery_status = Some((label.into(), "AVAILABLE (Connection Successful)".into(), "secure".into()));
                    } else {
                        alt_discovery_status = Some((label.into(), format!("UNKNOWN (HTTPS status {})", resp.status().as_u16()), "info".into()));
                    }
                }
                Err(e) => {
                    if label == "Plain HTTP Upgrade" {
                        alt_discovery_status = Some((label.into(), "SECURE (HTTP Connection Refused/Closed)".into(), "secure".into()));
                    } else {
                        let err_msg = e.to_string().to_lowercase();
                        if err_msg.contains("certificate") || err_msg.contains("ssl") || err_msg.contains("tls") {
                            alt_discovery_status = Some((label.into(), "NOT SECURE (Invalid Certificate)".into(), "insecure".into()));
                        } else {
                            alt_discovery_status = Some((label.into(), "UNAVAILABLE (Connection Refused/Closed)".into(), "info".into()));
                        }
                    }
                }
            }
        }
    }

    let mut transport_security = Vec::new();
    let final_is_https = final_url.starts_with("https://");

    if final_is_https {
        if let Some(ref err) = cert_error {
            transport_security.push(TransportInfo { label: "Connection Status".into(), value: "NOT SECURE (Invalid Certificate)".into(), status: "insecure".into() });
            transport_security.push(TransportInfo { label: "Certificate Error".into(), value: err.clone(), status: "insecure".into() });
        } else {
            transport_security.push(TransportInfo { label: "Connection Status".into(), value: "SECURE (HTTPS)".into(), status: "secure".into() });
            if !initial_is_https {
                transport_security.push(TransportInfo { label: "Redirection".into(), value: "SUCCESSFUL (HTTP -> HTTPS)".into(), status: "secure".into() });
            }
        }
    } else {
        transport_security.push(TransportInfo { label: "Connection Status".into(), value: "INSECURE (HTTP)".into(), status: "insecure".into() });
    }

    if let Some((label, value, status)) = alt_discovery_status {
        transport_security.push(TransportInfo { label: label.to_string(), value: value.to_string(), status: status.to_string() });
    }

    let (present_headers, missing_headers, mut warnings) = analyze_security_headers(&headers);

    if !final_is_https && cert_error.is_none() {
        warnings.push(Warning { header: "transport-security".into(), message: "CRITICAL: Connection is not encrypted.".into(), severity: "critical".into() });
    }
    if cert_error.is_some() {
        warnings.push(Warning { header: "transport-security".into(), message: "CRITICAL: SSL/TLS certificate validation failed.".into(), severity: "critical".into() });
    }

    let cors = analyze_cors(&headers, &mut warnings);
    let cookie_warnings = analyze_cookies(&headers);
    analyze_info_leaks(&headers, &mut warnings);

    let mut all_headers = HashMap::new();
    for (key, value) in headers.iter() {
        all_headers.insert(key.to_string(), value.to_str().unwrap_or("Non-ASCII").to_string());
    }

    ScanResult {
        url: opts.url,
        final_url,
        status_code,
        http_version,
        transport_security,
        cors,
        cookie_warnings,
        present_headers,
        missing_headers,
        warnings,
        all_headers,
        error: None,
    }
}

pub fn scan_file(content: &str) -> ScanResult {
    let mut headers = reqwest::header::HeaderMap::new();
    let mut status_code: u16 = 200;
    let mut http_version = "HTTP/1.1".to_string();

    let mut lines = content.lines();
    if let Some(first_line) = lines.next() {
        let trimmed = first_line.trim();
        if trimmed.to_uppercase().starts_with("HTTP/") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                http_version = parts[0].to_string();
                if let Ok(code) = parts[1].parse::<u16>() {
                    status_code = code;
                }
            }
        } else if let Some((k, v)) = trimmed.split_once(':') {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.trim().to_lowercase().as_bytes()),
                reqwest::header::HeaderValue::from_str(v.trim()),
            ) {
                headers.insert(name, val);
            }
        }
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() { break; }
        if let Some((k, v)) = trimmed.split_once(':') {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.trim().to_lowercase().as_bytes()),
                reqwest::header::HeaderValue::from_str(v.trim()),
            ) {
                headers.insert(name, val);
            }
        }
    }

    let (present_headers, missing_headers, mut warnings) = analyze_security_headers(&headers);
    analyze_info_leaks(&headers, &mut warnings);

    let mut all_headers = HashMap::new();
    for (key, value) in headers.iter() {
        all_headers.insert(key.to_string(), value.to_str().unwrap_or("Non-ASCII").to_string());
    }

    ScanResult {
        url: "(file)".into(),
        final_url: "(file)".into(),
        status_code,
        http_version,
        transport_security: vec![TransportInfo { label: "Source".into(), value: "Local File".into(), status: "info".into() }],
        cors: Vec::new(),
        cookie_warnings: Vec::new(),
        present_headers,
        missing_headers,
        warnings,
        all_headers,
        error: None,
    }
}

fn analyze_security_headers(headers: &reqwest::header::HeaderMap) -> (Vec<HeaderEntry>, Vec<HeaderEntry>, Vec<Warning>) {
    let mut present = Vec::new();
    let mut missing = Vec::new();
    let mut warnings = Vec::new();

    for &(header, desc) in SECURITY_HEADERS {
        match headers.get(header) {
            Some(value) => {
                let val_str = value.to_str().unwrap_or("").to_lowercase();
                present.push(HeaderEntry { name: header.into(), value: value.to_str().unwrap_or("Non-ASCII").into(), description: desc.into() });

                if header == "x-content-type-options" && val_str != "nosniff" {
                    warnings.push(Warning { header: header.into(), message: format!("Value is not 'nosniff': {}", val_str), severity: "warning".into() });
                }
                if header == "cache-control" && !val_str.contains("no-store") && !val_str.contains("no-cache") {
                    warnings.push(Warning { header: header.into(), message: "Doesn't contain 'no-store' or 'no-cache'.".into(), severity: "warning".into() });
                }
                if header == "strict-transport-security" {
                    if !val_str.contains("includesubdomains") {
                        warnings.push(Warning { header: header.into(), message: "Missing 'includeSubDomains'.".into(), severity: "warning".into() });
                    }
                    if !val_str.contains("max-age") {
                        warnings.push(Warning { header: header.into(), message: "Missing 'max-age'.".into(), severity: "warning".into() });
                    }
                }
                if header == "x-xss-protection" && val_str == "0" {
                    warnings.push(Warning { header: header.into(), message: "XSS protection explicitly disabled.".into(), severity: "warning".into() });
                }
            }
            None => {
                missing.push(HeaderEntry { name: header.into(), value: String::new(), description: desc.into() });
            }
        }
    }

    (present, missing, warnings)
}

fn analyze_cors(headers: &reqwest::header::HeaderMap, warnings: &mut Vec<Warning>) -> Vec<CorsInfo> {
    let mut cors = Vec::new();
    let acao = headers.get("access-control-allow-origin");
    let acac = headers.get("access-control-allow-credentials");
    let vary = headers.get("vary");

    if let Some(origin) = acao {
        let origin_str = origin.to_str().unwrap_or("");
        cors.push(CorsInfo { header: "access-control-allow-origin".into(), value: origin_str.into() });

        if origin_str == "*" {
            warnings.push(Warning { header: "access-control-allow-origin".into(), message: "Wildcard '*' origin is highly permissive.".into(), severity: "warning".into() });
        } else if origin_str == "null" {
            warnings.push(Warning { header: "access-control-allow-origin".into(), message: "'null' origin is exploitable via sandboxed iframes.".into(), severity: "warning".into() });
        } else {
            let vary_str = vary.and_then(|v| v.to_str().ok()).unwrap_or("").to_lowercase();
            let has_vary_origin = vary_str.split(',').any(|v| v.trim() == "origin");
            if let Some(v) = vary {
                cors.push(CorsInfo { header: "vary".into(), value: v.to_str().unwrap_or("").into() });
            }
            if !has_vary_origin {
                warnings.push(Warning { header: "vary".into(), message: "ACAO is specific origin but Vary doesn't include 'Origin'.".into(), severity: "warning".into() });
            }
        }

        if let Some(creds) = acac {
            let creds_str = creds.to_str().unwrap_or("");
            cors.push(CorsInfo { header: "access-control-allow-credentials".into(), value: creds_str.into() });
            if origin_str == "*" && creds_str == "true" {
                warnings.push(Warning { header: "cors-misconfiguration".into(), message: "CRITICAL: ACAO '*' with ACAC 'true'.".into(), severity: "critical".into() });
            }
        }
    }

    cors
}

fn analyze_cookies(headers: &reqwest::header::HeaderMap) -> Vec<CookieWarning> {
    let mut cookie_warnings = Vec::new();
    for cookie in headers.get_all("set-cookie") {
        let cookie_str = cookie.to_str().unwrap_or("");
        let mut missing = Vec::new();
        let lower = cookie_str.to_lowercase();
        if !lower.contains("httponly") { missing.push("HttpOnly"); }
        if !lower.contains("secure") { missing.push("Secure"); }
        if !lower.contains("samesite") { missing.push("SameSite"); }
        if !missing.is_empty() {
            cookie_warnings.push(CookieWarning {
                name: cookie_str.split('=').next().unwrap_or("Unknown").into(),
                missing_flags: missing.join(", "),
            });
        }
    }
    cookie_warnings
}

fn analyze_info_leaks(headers: &reqwest::header::HeaderMap, warnings: &mut Vec<Warning>) {
    for &(header, desc) in INFO_HEADERS {
        if let Some(value) = headers.get(header) {
            warnings.push(Warning {
                header: header.into(),
                message: format!("Information leak ({} = {}): {}", header, value.to_str().unwrap_or(""), desc),
                severity: "info".into(),
            });
        }
    }
}

fn error_result(url: &str, error: String) -> ScanResult {
    ScanResult {
        url: url.to_string(),
        final_url: String::new(),
        status_code: 0,
        http_version: String::new(),
        transport_security: Vec::new(),
        cors: Vec::new(),
        cookie_warnings: Vec::new(),
        present_headers: Vec::new(),
        missing_headers: Vec::new(),
        warnings: Vec::new(),
        all_headers: HashMap::new(),
        error: Some(error),
    }
}
