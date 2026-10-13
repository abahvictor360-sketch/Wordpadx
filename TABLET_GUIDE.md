# Wordpadx for Android Tablet & iPad

Wordpadx brings full-featured word processing and document design to tablets (Android tablets and iPad), running natively on the GPU via WebGPU and WebGL2.

---

## Deployment Options

### 1. Instant PWA Installation (Zero Build Required)
Both iPad (iPadOS) and Android tablets support installing Wordpadx directly as a fullscreen, offline-capable Progressive Web App:

- **On iPad (Safari):**
  1. Open the hosted Wordpadx URL in Safari.
  2. Tap the **Share** button (box with an arrow pointing up) in Safari's toolbar.
  3. Scroll down and tap **Add to Home Screen**.
  4. Wordpadx will appear as a standalone app on your iPad Home Screen and dock, running fullscreen without browser navigation bars.
  5. Supports **Split View**, **Slide Over**, **Stage Manager**, and **Apple Pencil**.

- **On Android Tablet (Chrome / Edge / Firefox):**
  1. Open the hosted Wordpadx URL.
  2. Tap the **three dots menu** (⋮) in the top-right corner.
  3. Tap **Install app** or **Add to Home screen**.
  4. Wordpadx will be installed into your tablet's app drawer and home screen.
  5. Supports Samsung DeX, multi-window, splitscreen, and stylus input (S-Pen).

---

### 2. Native Android Tablet App (`mobile/android/`)
A dedicated native Android container with hardware acceleration and deep Android OS integration:

- **Features:**
  - Hardware accelerated WebView rendering with WebGL / WebGPU.
  - Native file picker (`Intent.ACTION_GET_CONTENT` / Storage Access Framework) to open `.docx`, `.odt`, `.txt`, and images.
  - Automatic download handling to the tablet's `Downloads` folder for exported DOCX and PDF files.
  - Edge-to-edge display and display cutout support.

- **To Build Locally:**
  ```powershell
  # Windows PowerShell
  .\packaging\android\build.ps1
  ```
  ```bash
  # Linux / macOS
  ./packaging/android/build.sh
  ```
  Or open `mobile/android` in **Android Studio** and click **Run** or **Build APK**.

---

### 3. Native iPad App (`mobile/ios/`)
A dedicated native iPad container built with Swift and WebKit:

- **Features:**
  - Full iPadOS multitasking: **Split View**, **Slide Over**, and **Stage Manager**.
  - Document sharing via iOS Files app (`UIFileSharingEnabled` & `LSSupportsOpeningDocumentsInPlace`).
  - Bundled local offline assets with high-speed rendering.
  - Native Apple Pencil palm rejection and precision inking.

- **To Build Locally:**
  ```bash
  # On macOS with Xcode
  ./packaging/ios/build.sh
  ```
  Or open `mobile/ios/Wordpadx.xcodeproj` in **Xcode** and select your iPad device or simulator.

---

### 4. Automated Cloud Builds (GitHub Actions CI/CD)
The repository includes an automated workflow [`.github/workflows/build-mobile.yml`](.github/workflows/build-mobile.yml):
1. Every push to `main` automatically triggers:
   - WebAssembly compilation of Wordpadx.
   - Building the Android APK (`Wordpadx-Android-Tablet-Debug.apk`).
   - Building the iPad app (`Wordpadx-iPad-App`).
   - Deploying the live PWA to GitHub Pages.
2. Download ready-to-install builds directly from the **Actions** tab artifacts or releases.
