import java.util.Properties
import java.io.FileInputStream

plugins {
    id("com.android.application")
    id("kotlin-android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

// Rust build configuration
val rustProjectDir = file("../../rust")
val jniLibsDir = file("src/main/jniLibs")

tasks.register<Exec>("buildRustDebug") {
    description = "Build Rust library for Android (debug)"
    workingDir = rustProjectDir
    commandLine("cargo", "ndk", "-t", "arm64-v8a", "-o", jniLibsDir.absolutePath, "build")
}

tasks.register<Exec>("buildRustRelease") {
    description = "Build Rust library for Android (release)"
    workingDir = rustProjectDir
    commandLine("cargo", "ndk", "-t", "arm64-v8a", "-o", jniLibsDir.absolutePath, "build", "--release")
}

// Hook Rust build into Android build
tasks.whenTaskAdded {
    if (name == "mergeDebugJniLibFolders") {
        dependsOn("buildRustDebug")
    }
    if (name == "mergeReleaseJniLibFolders") {
        dependsOn("buildRustRelease")
    }
}

// Load keystore properties
val keystorePropertiesFile = rootProject.file("key.properties")
val keystoreProperties = Properties()
if (keystorePropertiesFile.exists()) {
    keystoreProperties.load(FileInputStream(keystorePropertiesFile))
}

android {
    namespace = "com.example.flutter_con"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_11.toString()
    }

    signingConfigs {
        create("release") {
            keyAlias = keystoreProperties["keyAlias"] as String?
            keyPassword = keystoreProperties["keyPassword"] as String?
            storeFile = keystoreProperties["storeFile"]?.let { file(it as String) }
            storePassword = keystoreProperties["storePassword"] as String?
        }
    }

    defaultConfig {
        applicationId = "com.example.flutter_con"
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.getByName("release")
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
}

flutter {
    source = "../.."
}
