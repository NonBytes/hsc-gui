const { invoke } = window.__TAURI__.core;

// --- Theme ---

const savedTheme = localStorage.getItem('hsc-theme') || 'dark';
document.documentElement.setAttribute('data-theme', savedTheme);
updateThemeIcon(savedTheme);

document.getElementById('theme-toggle').addEventListener('click', () => {
  const current = document.documentElement.getAttribute('data-theme');
  const next = current === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', next);
  localStorage.setItem('hsc-theme', next);
  updateThemeIcon(next);
});

function updateThemeIcon(theme) {
  document.getElementById('theme-icon').innerHTML = theme === 'dark' ? '&#9790;' : '&#9728;';
}

let scanResult = null;
let batchResults = null;
let fileResult = null;

document.querySelectorAll('.tab').forEach(tab => {
  tab.addEventListener('click', () => {
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(tc => tc.classList.remove('active'));
    tab.classList.add('active');
    document.getElementById(`tab-${tab.dataset.tab}`).classList.add('active');
    if (tab.dataset.tab === 'history') loadHistory();
  });
});

document.getElementById('toggle-headers-btn').addEventListener('click', () => {
  document.getElementById('custom-headers-section').classList.toggle('hidden');
});

// --- Single Scan ---

document.getElementById('scan-btn').addEventListener('click', scanSingle);
document.getElementById('url-input').addEventListener('keydown', e => {
  if (e.key === 'Enter') scanSingle();
});

async function scanSingle() {
  const url = document.getElementById('url-input').value.trim();
  if (!url) return;
  const followRedirects = document.getElementById('follow-redirects').checked;
  const headersText = document.getElementById('custom-headers').value.trim();
  const customHeaders = headersText ? headersText.split('\n').filter(h => h.trim()) : [];

  showLoading('scan');
  try {
    const result = await invoke('scan_url', { url, followRedirects, customHeaders });
    scanResult = result;
    showResults('scan', renderSingleResult(result));
    await invoke('save_to_history', { url, result });
  } catch (err) {
    showError('scan', err.toString());
  }
}

// --- Batch Scan ---

document.getElementById('batch-scan-btn').addEventListener('click', async () => {
  const text = document.getElementById('batch-urls').value.trim();
  if (!text) return;
  const urls = text.split('\n').map(u => u.trim()).filter(u => u);
  const followRedirects = document.getElementById('batch-follow-redirects').checked;

  showLoading('batch');
  const progressText = document.getElementById('batch-progress-text');
  const progressBar = document.getElementById('batch-progress-bar');
  const progressFill = progressBar.querySelector('.progress-fill');
  progressBar.classList.remove('hidden');
  progressFill.style.width = '0%';

  const results = [];
  for (let i = 0; i < urls.length; i++) {
    progressText.textContent = `Scanning ${i + 1} / ${urls.length}...`;
    progressFill.style.width = `${((i) / urls.length) * 100}%`;
    try {
      const result = await invoke('scan_url', { url: urls[i], followRedirects, customHeaders: [] });
      results.push(result);
      if (!result.error) {
        try { await invoke('save_to_history', { url: result.url, result }); } catch {}
      }
    } catch (err) {
      results.push({ url: urls[i], error: err.toString(), final_url: '', status_code: 0, http_version: '', transport_security: [], cors: [], cookie_warnings: [], present_headers: [], missing_headers: [], warnings: [], all_headers: {} });
    }
  }
  progressFill.style.width = '100%';
  batchResults = results;
  showResults('batch', renderBatchHTML(results));
});

document.getElementById('export-batch-json-btn').addEventListener('click', async () => {
  if (!batchResults) return;
  downloadText(JSON.stringify(batchResults, null, 2), 'hsc-batch-report.json', 'application/json');
});

document.getElementById('export-batch-md-btn').addEventListener('click', async () => {
  if (!batchResults) return;
  let md = '# Batch Security Headers Report\n\n';
  for (const result of batchResults) {
    if (result.error) {
      md += `## ${result.url}\n\nError: ${result.error}\n\n`;
    } else {
      try {
        const report = await invoke('export_report', { result, format: 'markdown' });
        md += report + '\n---\n\n';
      } catch {}
    }
  }
  downloadText(md, 'hsc-batch-report.md', 'text/markdown');
});

// --- File Import ---

document.getElementById('browse-file-btn').addEventListener('click', async () => {
  try {
    const selected = await window.__TAURI__.dialog.open({
      multiple: false,
      filters: [{ name: 'Text Files', extensions: ['txt', 'headers', 'http', 'log', '*'] }],
    });
    if (selected) {
      const content = await invoke('read_file', { path: selected });
      document.getElementById('file-content').value = content;
    }
  } catch (err) { console.error(err); }
});

document.getElementById('file-scan-btn').addEventListener('click', async () => {
  const content = document.getElementById('file-content').value.trim();
  if (!content) return;
  showLoading('file');
  try {
    const result = await invoke('scan_file', { content });
    fileResult = result;
    showResults('file', renderSingleResult(result));
    const label = document.getElementById('file-label').value.trim();
    const historyUrl = label || `file: ${content.split('\n')[0].substring(0, 80)}`;
    await invoke('save_to_history', { url: historyUrl, result });
  } catch (err) {
    showError('file', err.toString());
  }
});

// --- Export (scan + file) ---

document.addEventListener('click', async (e) => {
  if (e.target.classList.contains('export-json-btn')) {
    const result = e.target.closest('#tab-scan') ? scanResult : fileResult;
    if (!result) return;
    try {
      const report = await invoke('export_report', { result, format: 'json' });
      downloadText(report, 'hsc-report.json', 'application/json');
    } catch (err) { console.error(err); }
  }
  if (e.target.classList.contains('export-md-btn') || e.target.classList.contains('file-export-md-btn')) {
    const result = e.target.closest('#tab-scan') ? scanResult : fileResult;
    if (!result) return;
    try {
      const report = await invoke('export_report', { result, format: 'markdown' });
      downloadText(report, 'hsc-report.md', 'text/markdown');
    } catch (err) { console.error(err); }
  }
  if (e.target.classList.contains('file-export-json-btn')) {
    if (!fileResult) return;
    try {
      const report = await invoke('export_report', { result: fileResult, format: 'json' });
      downloadText(report, 'hsc-report.json', 'application/json');
    } catch (err) { console.error(err); }
  }
});

// --- Compare ---

document.querySelectorAll('.compare-headers-toggle').forEach(btn => {
  btn.addEventListener('click', () => {
    document.getElementById(btn.dataset.target).classList.toggle('hidden');
  });
});

document.getElementById('compare-file-toggle-headers-btn').addEventListener('click', () => {
  document.getElementById('compare-file-custom-headers-section').classList.toggle('hidden');
});

document.querySelectorAll('.compare-mode').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.compare-mode').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.compare-mode-content').forEach(c => c.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById(`compare-mode-${btn.dataset.mode}`).classList.add('active');
    document.getElementById('compare-results').classList.add('hidden');
    if (btn.dataset.mode === 'history') loadCompareHistoryPicker();
  });
});

// --- Compare vs History picker ---

let compareHistoryCache = [];
let selectedHistoryEntry = null;

async function loadCompareHistoryPicker() {
  try {
    compareHistoryCache = await invoke('get_history');
    renderCompareHistPicker(compareHistoryCache);
  } catch (err) { console.error(err); }
}

function renderCompareHistPicker(entries) {
  const query = (document.getElementById('compare-hist-search').value || '').toLowerCase();
  const filtered = query ? entries.filter(e => e.url.toLowerCase().includes(query)) : entries;
  const list = document.getElementById('compare-hist-list');
  if (!filtered.length) {
    list.innerHTML = `<div class="empty-state" style="padding:16px">No history entries.</div>`;
    return;
  }
  list.innerHTML = filtered.map(e => {
    const score = e.result.error ? 0 : calculateScore(e.result);
    const cls = score >= 70 ? 'status-secure' : score >= 40 ? 'status-warning' : 'status-insecure';
    const sel = selectedHistoryEntry && selectedHistoryEntry.id === e.id ? ' selected' : '';
    return `<div class="compare-hist-item${sel}" data-hist-id="${esc(e.id)}">
      <div class="hist-item-url">${esc(e.url)}</div>
      <div class="hist-item-meta"><span class="${cls}">Score: ${score}</span><span>${formatDate(e.timestamp)}</span></div>
    </div>`;
  }).join('');

  list.querySelectorAll('.compare-hist-item').forEach(el => {
    el.addEventListener('click', () => {
      selectedHistoryEntry = compareHistoryCache.find(e => e.id === el.dataset.histId) || null;
      list.querySelectorAll('.compare-hist-item').forEach(i => i.classList.remove('selected'));
      el.classList.add('selected');
    });
  });
}

document.getElementById('compare-hist-search').addEventListener('input', () => {
  renderCompareHistPicker(compareHistoryCache);
});

document.getElementById('compare-hist-btn').addEventListener('click', async () => {
  const url = document.getElementById('compare-hist-url').value.trim();
  if (!url) return;
  if (!selectedHistoryEntry) {
    alert('Please select a history entry to compare against.');
    return;
  }
  const followRedirects = document.getElementById('compare-hist-follow-redirects').checked;
  const headersText = document.getElementById('compare-hist-custom-headers').value.trim();
  const customHeaders = headersText ? headersText.split('\n').filter(h => h.trim()) : [];
  const histLabel = `${selectedHistoryEntry.url} (${formatDate(selectedHistoryEntry.timestamp)})`;

  document.getElementById('compare-loading').classList.remove('hidden');
  document.getElementById('compare-results').classList.add('hidden');

  try {
    const liveResult = await invoke('scan_url', { url, followRedirects, customHeaders });
    showCompareResults(liveResult, selectedHistoryEntry.result, liveResult.final_url || url, histLabel);
  } catch (err) {
    document.getElementById('compare-loading').classList.add('hidden');
    document.getElementById('compare-results').classList.remove('hidden');
    document.getElementById('compare-summary').innerHTML = `<div class="error-message">${esc(err.toString())}</div>`;
  }
});

document.getElementById('compare-browse-btn').addEventListener('click', async () => {
  try {
    const selected = await window.__TAURI__.dialog.open({
      multiple: false,
      filters: [{ name: 'Text Files', extensions: ['txt', 'headers', 'http', 'log', '*'] }],
    });
    if (selected) {
      const content = await invoke('read_file', { path: selected });
      document.getElementById('compare-file-content').value = content;
    }
  } catch (err) { console.error(err); }
});

function showCompareResults(resultA, resultB, labelA, labelB) {
  const scoreA = resultA.error ? 0 : calculateScore(resultA);
  const scoreB = resultB.error ? 0 : calculateScore(resultB);
  const clsA = scoreA >= 70 ? 'score-good' : scoreA >= 40 ? 'score-ok' : 'score-bad';
  const clsB = scoreB >= 70 ? 'score-good' : scoreB >= 40 ? 'score-ok' : 'score-bad';
  const diff = scoreA - scoreB;
  const diffText = diff > 0 ? `A wins by +${diff}` : diff < 0 ? `B wins by +${Math.abs(diff)}` : 'Tied';

  document.getElementById('compare-summary').innerHTML = `
    <div class="compare-score">
      <div class="score-label">${esc(labelA)}</div>
      <div class="score-value ${clsA}">${scoreA}</div>
    </div>
    <div class="compare-arrow">${diffText}</div>
    <div class="compare-score">
      <div class="score-label">${esc(labelB)}</div>
      <div class="score-value ${clsB}">${scoreB}</div>
    </div>`;

  document.getElementById('compare-panel-a').innerHTML = resultA.error
    ? `<div class="error-message">${esc(resultA.error)}</div>`
    : renderSingleResult(resultA);
  document.getElementById('compare-panel-b').innerHTML = resultB.error
    ? `<div class="error-message">${esc(resultB.error)}</div>`
    : renderSingleResult(resultB);

  document.getElementById('compare-loading').classList.add('hidden');
  document.getElementById('compare-results').classList.remove('hidden');
}

document.getElementById('compare-btn').addEventListener('click', async () => {
  const urlA = document.getElementById('compare-url-a').value.trim();
  const urlB = document.getElementById('compare-url-b').value.trim();
  if (!urlA || !urlB) return;
  const followRedirects = document.getElementById('compare-follow-redirects').checked;
  const parseHeaders = id => (document.getElementById(id).value.trim()).split('\n').filter(h => h.trim());
  const customHeadersA = parseHeaders('compare-custom-headers-a');
  const customHeadersB = parseHeaders('compare-custom-headers-b');

  document.getElementById('compare-loading').classList.remove('hidden');
  document.getElementById('compare-results').classList.add('hidden');

  try {
    const [resultA, resultB] = await Promise.all([
      invoke('scan_url', { url: urlA, followRedirects, customHeaders: customHeadersA }),
      invoke('scan_url', { url: urlB, followRedirects, customHeaders: customHeadersB }),
    ]);
    showCompareResults(resultA, resultB, resultA.final_url || urlA, resultB.final_url || urlB);
  } catch (err) {
    document.getElementById('compare-loading').classList.add('hidden');
    document.getElementById('compare-results').classList.remove('hidden');
    document.getElementById('compare-summary').innerHTML = `<div class="error-message">${esc(err.toString())}</div>`;
  }
});

document.getElementById('compare-file-btn').addEventListener('click', async () => {
  const url = document.getElementById('compare-file-url').value.trim();
  const content = document.getElementById('compare-file-content').value.trim();
  if (!url || !content) return;
  const followRedirects = document.getElementById('compare-file-follow-redirects').checked;
  const label = document.getElementById('compare-file-label').value.trim() || 'File Headers';
  const fileHeadersText = document.getElementById('compare-file-custom-headers').value.trim();
  const fileCustomHeaders = fileHeadersText ? fileHeadersText.split('\n').filter(h => h.trim()) : [];

  document.getElementById('compare-loading').classList.remove('hidden');
  document.getElementById('compare-results').classList.add('hidden');

  try {
    const [resultA, resultB] = await Promise.all([
      invoke('scan_url', { url, followRedirects, customHeaders: fileCustomHeaders }),
      invoke('scan_file', { content }),
    ]);
    showCompareResults(resultA, resultB, resultA.final_url || url, label);
  } catch (err) {
    document.getElementById('compare-loading').classList.add('hidden');
    document.getElementById('compare-results').classList.remove('hidden');
    document.getElementById('compare-summary').innerHTML = `<div class="error-message">${esc(err.toString())}</div>`;
  }
});

function downloadText(text, filename, mime) {
  const blob = new Blob([text], { type: mime });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
  URL.revokeObjectURL(a.href);
}

// --- History ---

document.getElementById('clear-history-btn').addEventListener('click', async () => {
  if (!confirm('Clear all scan history?')) return;
  try { await invoke('clear_history'); loadHistory(); } catch (err) { console.error(err); }
});

let activeHistoryId = null;
let historyCache = [];

document.getElementById('history-search').addEventListener('input', () => {
  renderHistoryList(historyCache);
});

async function loadHistory() {
  try {
    historyCache = await invoke('get_history');
    activeHistoryId = null;
    renderHistoryList(historyCache);
  } catch (err) { console.error(err); }
}

function renderHistoryList(history) {
  const container = document.getElementById('history-list');
  const query = (document.getElementById('history-search').value || '').toLowerCase();
  const filtered = query ? history.filter(e => e.url.toLowerCase().includes(query)) : history;

  if (!filtered.length) {
    container.innerHTML = `<div class="empty-state">${query ? 'No matching results.' : 'No scan history yet.'}</div>`;
    return;
  }

  const groups = new Map();
  for (const entry of filtered) {
    const key = entry.url;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(entry);
  }

  let html = '';
  for (const [url, entries] of groups) {
    const latest = entries[0];
    const score = latest.result.error ? 0 : calculateScore(latest.result);
    const cls = score >= 70 ? 'status-secure' : score >= 40 ? 'status-warning' : 'status-insecure';
    const countBadge = entries.length > 1 ? `<span class="history-count">${entries.length} scans</span>` : '';

    html += `
      <div class="history-group" data-url="${esc(url)}">
        <div class="history-entry" data-id="${esc(latest.id)}">
          <div>
            <div class="history-url">${esc(url)} ${countBadge}</div>
            <div class="history-meta">
              <span>Status: ${latest.result.status_code}</span>
              <span class="${cls}">Score: ${score}</span>
              <span>${formatDate(latest.timestamp)}</span>
            </div>
          </div>
          <button class="history-delete" onclick="event.stopPropagation(); deleteHistoryEntry('${esc(latest.id)}')" title="Delete">&times;</button>
        </div>
      </div>`;
  }
  container.innerHTML = html;

  container.querySelectorAll('.history-entry').forEach(el => {
    el.addEventListener('click', () => {
      const group = el.closest('.history-group');
      const url = group.dataset.url;
      const entries = groups.get(url);
      const entry = entries.find(h => h.id === el.dataset.id) || entries[0];
      if (!entry) return;

      const existing = container.querySelector('.history-detail-inline');

      if (activeHistoryId === entry.id) {
        if (existing) existing.remove();
        el.classList.remove('selected');
        activeHistoryId = null;
        return;
      }

      if (existing) existing.remove();
      container.querySelectorAll('.history-entry').forEach(e => e.classList.remove('selected'));

      el.classList.add('selected');
      activeHistoryId = entry.id;

      const detail = document.createElement('div');
      detail.className = 'history-detail-inline';

      let detailHTML = '';
      if (entries.length > 1) {
        detailHTML += `<div class="history-versions"><span class="versions-label">Scan history:</span>`;
        entries.forEach((e, i) => {
          const s = e.result.error ? 0 : calculateScore(e.result);
          const c = s >= 70 ? 'status-secure' : s >= 40 ? 'status-warning' : 'status-insecure';
          const active = e.id === entry.id ? ' version-active' : '';
          detailHTML += `<button class="version-btn${active}" data-version-id="${esc(e.id)}"><span class="${c}">${s}</span> ${formatDate(e.timestamp)}</button>`;
        });
        detailHTML += `</div>`;
      }
      detailHTML += `<div class="history-detail-content">${renderSingleResult(entry.result)}</div>`;
      detail.innerHTML = detailHTML;

      group.insertAdjacentElement('afterend', detail);

      detail.querySelectorAll('.version-btn').forEach(btn => {
        btn.addEventListener('click', () => {
          const vEntry = entries.find(e => e.id === btn.dataset.versionId);
          if (!vEntry) return;
          activeHistoryId = vEntry.id;
          detail.querySelectorAll('.version-btn').forEach(b => b.classList.remove('version-active'));
          btn.classList.add('version-active');
          detail.querySelector('.history-detail-content').innerHTML = renderSingleResult(vEntry.result);
        });
      });

      detail.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    });
  });
}

window.deleteHistoryEntry = async function(id) {
  try { await invoke('delete_history_entry', { id }); loadHistory(); } catch (err) { console.error(err); }
};

// --- UI Helpers ---

function showLoading(tab) {
  document.getElementById(`${tab}-loading`).classList.remove('hidden');
  document.getElementById(`${tab}-results`).classList.add('hidden');
}

function showResults(tab, html) {
  document.getElementById(`${tab}-loading`).classList.add('hidden');
  const container = document.getElementById(`${tab}-results`);
  container.classList.remove('hidden');
  container.querySelector('.results-content').innerHTML = html;
}

function showError(tab, msg) {
  document.getElementById(`${tab}-loading`).classList.add('hidden');
  const container = document.getElementById(`${tab}-results`);
  container.classList.remove('hidden');
  container.querySelector('.results-content').innerHTML = `<div class="error-message">${esc(msg)}</div>`;
}

function esc(str) {
  const el = document.createElement('div');
  el.textContent = str || '';
  return el.innerHTML;
}

function formatDate(iso) {
  try { return new Date(iso).toLocaleString(); } catch { return iso; }
}

function calculateScore(result) {
  const total = 11;
  const present = result.present_headers.length;
  const criticals = result.warnings.filter(w => w.severity === 'critical').length;
  return Math.max(0, Math.round((present / total) * 100) - criticals * 15);
}

// --- Rendering ---

function renderSingleResult(result) {
  const score = calculateScore(result);
  const scoreClass = score >= 70 ? 'score-good' : score >= 40 ? 'score-ok' : 'score-bad';

  let html = `<div class="score-badge ${scoreClass}">Security Score: ${score}/100</div>`;

  html += `<div class="meta-info">
    <span><strong>URL:</strong> ${esc(result.final_url)}</span>
    <span><strong>Status:</strong> ${result.status_code}</span>
    <span><strong>HTTP:</strong> ${esc(result.http_version)}</span>
  </div>`;

  html += renderSection('blue', 'Transport Security', result.transport_security.map(t =>
    `<div class="result-item"><span class="result-label">${esc(t.label)}</span><span class="result-value status-${t.status}">${esc(t.value)}</span></div>`
  ));

  html += renderSection('cyan', 'CORS Configuration',
    result.cors.length === 0
      ? ['<div class="result-item"><span class="status-info">No CORS headers (same-origin default)</span></div>']
      : result.cors.map(c => `<div class="result-item"><span class="result-label">${esc(c.header)}</span><span class="result-value">${esc(c.value)}</span></div>`)
  );

  html += renderSection('magenta', 'Cookie Security',
    result.cookie_warnings.length === 0
      ? ['<div class="result-item"><span class="status-info">No cookie issues</span></div>']
      : result.cookie_warnings.map(c => `<div class="result-item"><span class="result-label header-missing">${esc(c.name)}</span><span>Missing: ${esc(c.missing_flags)}</span></div>`)
  );

  html += renderSection('green', `Security Headers Present (${result.present_headers.length})`,
    result.present_headers.length === 0
      ? ['<div class="result-item"><span class="status-info">None</span></div>']
      : result.present_headers.map(h => `<div class="result-item"><span class="result-label header-present">${esc(h.name)}</span><span class="result-value">${esc(h.value)}</span></div>`)
  );

  html += renderSection('red', `Missing Security Headers (${result.missing_headers.length})`,
    result.missing_headers.length === 0
      ? ['<div class="result-item"><span class="header-present">None — excellent!</span></div>']
      : result.missing_headers.map(h => `<div class="result-item"><span class="result-label header-missing">${esc(h.name)}</span><span class="header-desc">${esc(h.description)}</span></div>`)
  );

  html += renderSection('yellow', `Warnings & Info Leaks (${result.warnings.length})`,
    result.warnings.length === 0
      ? ['<div class="result-item"><span class="header-present">None — good job!</span></div>']
      : result.warnings.map(w => `<div class="result-item"><span class="result-label severity-${w.severity}">${esc(w.header)}</span><span class="severity-${w.severity}">${esc(w.message)}</span></div>`)
  );

  const headerEntries = Object.entries(result.all_headers);
  if (headerEntries.length > 0) {
    html += `<div class="result-section">
      <h3><span class="section-icon blue"></span>All Response Headers (${headerEntries.length})</h3>
      <table class="headers-table">${headerEntries.map(([k, v]) => `<tr><td>${esc(k)}</td><td>${esc(v)}</td></tr>`).join('')}</table>
    </div>`;
  }

  return html;
}

function renderResults(result) {
  if (result.error) {
    showError('scan', result.error);
    return;
  }
  showResults('scan', renderSingleResult(result));
}

function renderSection(color, title, items) {
  return `<div class="result-section"><h3><span class="section-icon ${color}"></span>${title}</h3>${items.join('')}</div>`;
}

function renderBatchHTML(results) {
  let html = `<h3 style="margin-bottom:12px">Batch Results (${results.length} URLs)</h3>`;
  results.forEach((result, i) => {
    const score = result.error ? 0 : calculateScore(result);
    const cls = score >= 70 ? 'status-secure' : score >= 40 ? 'status-warning' : 'status-insecure';
    html += `
      <div class="batch-result-header" data-batch-idx="${i}">
        <span>${esc(result.url)} ${result.error ? '(Error)' : `— Status ${result.status_code}`}</span>
        <span class="${cls}">Score: ${score}</span>
      </div>
      <div class="batch-result-body" id="batch-${i}">
        ${result.error
          ? `<div class="error-message">${esc(result.error)}</div>`
          : renderSingleResult(result)}
      </div>`;
  });

  setTimeout(() => {
    document.querySelectorAll('.batch-result-header').forEach(el => {
      el.addEventListener('click', () => {
        el.nextElementSibling.classList.toggle('open');
      });
    });
  }, 0);

  return html;
}
