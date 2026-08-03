# Kotisatama Android

> **Android-sovellus elää Katselin-päärepossa.**  
> Repo: https://github.com/Mikko-Huuskonen-Pro/Katselin  
> Polku: `android/apk/`, orkestrointi: `scripts/build-android.sh`  
> Suunnitelma: sisaruscheckoutissa `../Katselin/docs/REPO-JAKO-SUUNNITELMA.md`

Servo EGL / JNI pysyy tässä forkissa (`ports/servoshell/egl/android/`).
Gradle-projekti **ei** ole enää tämän repon lähdepuussa.

## Paikallinen junction (mach tarvitsee polun)

`mach build --target …-android` odottaa edelleen polkua
`support/android/apk`. Paikallisesti se on **Windows-junction**
(tai WSL-symlink) Katselin-apk:hon:

```
support/android/apk  →  ../Katselin/android/apk
```

Luo / korjaa:

```powershell
# Windows (repo juuresta)
.\scripts\link-android-apk.ps1
```

```bash
# WSL / Linux
./scripts/link-android-apk.sh
```

Junction **ei** kuulu gittiin (`support/android/apk/` on gitignoressa).
Kloonin jälkeen aja linkitysskripti kerran.

## Build

Suosi Katselin-orkestroijaa:

```bash
cd ../Katselin
./scripts/build-android.sh --skip-bootstrap --target x86_64-linux-android
```

Täydet ohjeet: `../Katselin/android/README.md`
