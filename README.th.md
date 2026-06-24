# Headers Security Checker (HSC GUI)

🇺🇸 [English](README.md)

แอปพลิเคชันสำหรับตรวจสอบ วิเคราะห์ และเปรียบเทียบ HTTP Response Headers เพื่อประเมินความปลอดภัยและการตั้งค่าความปลอดภัยของเว็บเซิร์ฟเวอร์และแอปพลิเคชันของคุณ ตัวแอปถูกพัฒนาขึ้นด้วย **Tauri v2** และ **Rust** ในส่วนของ Backend และเขียนหน้าตาด้วย **HTML5 / CSS3 / JavaScript** (Vanilla) ทำให้มีขนาดกะทัดรัด ทำงานได้รวดเร็ว และประหยัดทรัพยากรเครื่อง

---

## ✨ ฟีเจอร์เด่น (Features)

- **🔍 Single URL Scan**: วิเคราะห์ความปลอดภัยของ HTTP Headers สำหรับ URL เดียว สามารถตั้งค่าให้ติดตามการเปลี่ยนเส้นทาง (Follow Redirects) และเพิ่ม Custom HTTP Headers ในการร้องขอได้ (เช่น Authorization token)
- **📦 Batch URL Scan**: สแกนและวิเคราะห์ความปลอดภัยของหลาย URL พร้อมกันแบบ Concurrently ช่วยประหยัดเวลาเมื่อต้องสแกนตรวจสอบหลายระบบพร้อมกัน
- **📄 Raw Header Import**: วิเคราะห์ความปลอดภัยโดยการคัดลอก HTTP Headers แบบดิบมาวาง หรือนำเข้าจากไฟล์ Text โดยตรงโดยไม่ต้องสแกนผ่านเน็ตเวิร์กจริง
- **⚔️ Side-by-Side Comparison (ระบบเปรียบเทียบ)**:
  - **URL vs. URL**: เปรียบเทียบความแตกต่างของการตั้งค่า Headers ระหว่างสองระบบ (เช่น Staging กับ Production)
  - **URL vs. File/Raw**: เปรียบเทียบ Headers ของ Live Website กับไฟล์อ้างอิงหรือข้อมูลดิบที่ป้อนเข้ามา
- **📜 Scan History**: ระบบบันทึกประวัติการสแกนย้อนหลัง เพื่อให้สามารถเรียกดูผลการวิเคราะห์ก่อนหน้าได้อย่างรวดเร็ว
- **📊 Export Reports**: สามารถส่งออกผลลัพธ์การสแกนออกมาเป็นไฟล์ **JSON** หรือไฟล์ **Markdown** (.md) ที่จัดรูปแบบสวยงามได้
- **🌗 Responsive Dark/Light UI**: มีปุ่มสำหรับสลับโหมดมืด (Dark Mode) และโหมดสว่าง (Light Mode) เพื่อความสบายตาในการใช้งาน

---

## 🛡️ หัวข้อการตรวจสอบความปลอดภัยของ Headers

ระบบจะทำการตรวจสอบและประเมินผลในด้านต่าง ๆ ดังนี้:
1. **Transport Security (ความปลอดภัยในการรับส่งข้อมูล)**: การตรวจสอบการตั้งค่า HSTS (`Strict-Transport-Security`)
2. **CORS Policies (นโยบาย Cross-Origin)**: ความปลอดภัยของนโยบายการเรียกใช้ทรัพยากรต่างโดเมน
3. **Cookie Security (ความปลอดภัยของคุกกี้)**: ตรวจสอบและแจ้งเตือนหากคุกกี้ขาดแฟล็กสำคัญ เช่น `Secure`, `HttpOnly`, และ `SameSite`
4. **Vulnerabilities & Leaks (การรั่วไหลของข้อมูล)**: ตรวจจับข้อมูลสำคัญที่อาจเปิดเผยผ่าน Response Headers เช่น เวอร์ชันของเซิร์ฟเวอร์หรือภาษาที่เขียน (`Server`, `X-Powered-By`)
5. **Missing Best-Practice Headers**: แจ้งเตือนเมื่อขาดแคลน Headers ป้องกันภัยคุกคามมาตรฐาน เช่น `Content-Security-Policy`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` และอื่น ๆ

---

## 🛠️ เทคโนโลยีที่ใช้พัฒนา (Technology Stack)

- **Backend**: 
  - [Rust](https://www.rust-lang.org/) (ใช้งานร่วมกับ Tauri v2)
  - ไลบรารี `reqwest` และ `tokio` สำหรับสแกนความปลอดภัยแบบอะซิงโครนัสที่มีประสิทธิภาพสูง
  - ไลบรารี `serde` และ `serde_json` สำหรับการจัดการข้อมูล JSON
- **Frontend**:
  - Vanilla HTML5 / modern CSS3
  - Custom JavaScript (ใช้งานแบบ Static Web ในตัวแอป)
- **CI/CD**:
  - GitHub Actions สำหรับบิลด์ไฟล์ติดตั้งข้ามแพลตฟอร์มโดยอัตโนมัติ (`.dmg`, `.deb`, `.AppImage`, `.msi`, `.exe`) และอัปโหลดไปยัง GitHub Releases

---

## 🚀 เริ่มต้นใช้งาน (Getting Started)

### 📋 สิ่งที่ต้องติดตั้งก่อน (Prerequisites)

กรุณาตรวจสอบว่าคุณได้ติดตั้งเครื่องมือเหล่านี้บนเครื่องของคุณเรียบร้อยแล้ว:
- **Rust**: ตัวติดตั้ง [Rustup](https://rustup.rs/) (เวอร์ชัน Stable)
- **Tauri Prerequisites**: ติดตั้งแพ็กเกจเสริมตามระบบปฏิบัติการที่คุณใช้งานตามคำแนะนำของ Tauri:
  - [Tauri Prerequisites Guide](https://v2.tauri.app/start/prerequisites/)

### 📦 ขั้นตอนการรันและการบิลด์

1. โคลนคลังโค้ดนี้ลงเครื่อง:
   ```bash
   git clone https://github.com/NonBytes/hsc-gui.git
   cd hsc-gui
   ```

2. ติดตั้ง Tauri CLI บนเครื่องแบบ Global (หากยังไม่ได้ติดตั้ง):
   ```bash
   cargo install tauri-cli --version "^2"
   ```

3. รันโปรเจกต์ในโหมดพัฒนา (Development Mode):
   ```bash
   cargo tauri dev
   ```

4. บิลด์ตัวติดตั้งสำหรับใช้งานจริง (Production Build):
   ```bash
   cargo tauri build
   ```
   *ไฟล์ตัวติดตั้งสำหรับระบบปฏิบัติการของคุณจะถูกเก็บไว้ที่ `src-tauri/target/release/bundle/`*

---

## 🤖 กระบวนการ CI/CD (Workflow)

ตัวโปรเจกต์ได้มีการตั้งค่า Workflow ของ GitHub Actions เอาไว้ที่ `.github/workflows/release.yml` ซึ่งจะทำงานเมื่อมีการสั่งสร้าง Release เพื่อบิลด์ตัวติดตั้งข้ามแพลตฟอร์มโดยอัตโนมัติ

### วิธีการเริ่มรันกระบวนการ Release:
- **สั่งรันด้วยตัวเอง (Manual Trigger - แนะนำ)**: เข้าไปที่แท็บ **Actions** บนหน้าเว็บ GitHub -> เลือก workflow **Release** -> คลิกปุ่ม **Run workflow** จากนั้นกรอกเวอร์ชันของ Release (เช่น `v0.1.1`) ที่ต้องการ
- **สั่งรันผ่าน Git Tag**: สร้างและ Push Tag ที่ขึ้นต้นด้วยตัว `v` (เช่น `v0.1.1`):
  ```bash
  git tag v0.1.1
  git push origin v0.1.1
  ```
