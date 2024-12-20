[![](https://jitpack.io/v/functionland/wnfs-build-aar.svg)](https://jitpack.io/#functionland/wnfs-build-aar)


# WNFS-lib For Android

Webnative Filesystem(WNFS) wrapper for Android.

## Usage

Exposed endpoint: mkdir, writeFile, writeFileFromPath, readFile, readFileToPath, readFilestreamToPath, rm, cp, mv

- Library is already packaged and published on Jitpack and ready to be used in Android applications (Java, Kotlin). Please checkout the AppMock for all usage examples: https://github.com/functionland/wnfs-android/blob/main/appmock/src/androidTest/java/land/fx/app/WNFSTest.kt

- .aar files are available here that can be imported in ny framework: https://github.com/functionland/wnfs-build-aar

- It can be used in other frameworks, suchas React Native or Flutter as welll. Checkout react-native-fula for an exmaple of how to use this library in react-native and how to transfer the DAG with libp2p: https://github.com/functionland/react-native-fula


# Manual Build

## Prerequisites

Use Ubuntu (on Windows it complains about openssl)

- Kotlin toolchain
- Android SDK + NDK (latest)
- python3 and Python command available
- Cargo and CMake
- java
- gradle
- rustup target add armv7-linux-androideabi aarch64-linux-android i686-linux-android x86_64-linux-android

## Common Errors
- Make sure the build.gradle shows the correct ndk version you have if you see:
  
```bash
[CXX1104] NDK from ndk.dir at /.../ndk/25.2.9519653 had version [25....] which disagrees with android.ndkVersion [25.1.8937393]
```

- Make sure you define a local.properties file in the project root with the below line in it if you see NDK not found:
- 
```bash
ndk.dir=/paht-to-ndk/ndk/25.....
```

## Debug

Make sure you have switch to `debug` profile in cargo config, which could be found at `lib/build.gradle` 

Run the command to build

```sh
./gradlew lib:assemble
```

Connect to a device or setup an AVD and check the functionality.

```sh
./gradlew appmock:connectedCheck
```

## Build

Before make a release build, ensure you have set `profile = "release"` in cargo config.

```sh
./gradlew lib:assemble
```

The generated release build is `lib/build/outputs/aar/lib-release.aar`

## Publish New Version

Ensure you have committed your changes.

```sh
./gradlew release
```

Then simply push to the repo.


## Roadmap

Please note the following might not be done in order:

- [x] Initial version with all functions included
- [x] Add WNFS tree encryption key generation from an input (deterministically)
- [x] add error catching
- [x] add metadata to ls and make it array
- [x] Improve read function to use a stream. ( :100: v1 Release here )
- [x] remove dependancy to custom version of wnfs
