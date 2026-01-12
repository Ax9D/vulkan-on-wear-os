# Vulkan on WearOS

A Rust demo of Vulkan rendering on WearOS, powered by the [Hikari](https://github.com/Ax9D/Hikari) engine and [android-activity](https://github.com/rust-mobile/android-activity) developed over a weekend.

Demo running on my Oneplus Watch 2:

https://github.com/user-attachments/assets/3618dfdc-5464-43f8-852a-e78bfe67f5da


## Prerequisites

### Android SDK & NDK

Install the Android SDK, platform tools, and NDK using [Google's official command-line tooling](https://developer.android.com/studio#command-tools).

**Required components:**
- Android SDK
- Platform Tools (adb)
- Android NDK 25.2.9519653

**Environment variables:**
- `ANDROID_HOME` or `ANDROID_SDK_ROOT`
- `ANDROID_NDK_HOME` (pointing to NDK 25.2.9519653)

### Cargo APK

Install the APK build tool:
```bash
cargo install cargo-apk
```

## Connecting a WearOS Watch

1. Enable developer mode on the watch:
   ```
   Settings → About → Software info → Tap Build number 7 times
   ```

2. Enable debugging:
   ```
   Settings → Developer options → ADB debugging
   ```

3. Connect the watch using either USB or wireless ADB.

List connected devices:

```bash
adb devices
```

When prompted on the watch, allow the debugging connection.

## Verify Vulkan Support

```bash
adb shell pm list features | grep vulkan
```

## Building and Running on Watch

```bash
cargo apk run
```

## Debugging

```bash
adb logcat
```
