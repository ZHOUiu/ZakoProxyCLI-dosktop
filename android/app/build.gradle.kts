plugins {
    id("com.android.application")
}

val webDist: File = file("../../ZakoProxyCLI-0.13.3/web/dist")

android {
    namespace = "io.zakoai.proxy"
    compileSdk = 35
    buildToolsVersion = "35.0.0"

    defaultConfig {
        applicationId = "io.zakoai.proxy"
        minSdk = 26
        targetSdk = 35
        versionCode = 1303
        versionName = "0.13.3"
    }

    signingConfigs {
        create("local") {
            val store = File(System.getProperty("user.home"), ".android/debug.keystore")
            storeFile = store
            storePassword = "android"
            keyAlias = "androiddebugkey"
            keyPassword = "android"
        }
    }

    buildTypes {
        getByName("debug") {
            signingConfig = signingConfigs.getByName("local")
        }
        getByName("release") {
            isMinifyEnabled = false
            isShrinkResources = false
            signingConfig = signingConfigs.getByName("local")
        }
    }

    sourceSets {
        getByName("main") {
            assets.srcDirs(file("src/main/assets"), layout.buildDirectory.dir("staged-web"))
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    packaging {
        // Native libraries are loaded straight out of the APK (extractNativeLibs=false).
        jniLibs {
            useLegacyPackaging = false
            // Strip is done by the NDK toolchain during cross-compilation already.
            keepDebugSymbols += "**/libzakoai.so"
        }
    }

    androidResources {
        noCompress += listOf("bin")
    }

    lint {
        abortOnError = false
    }
}

dependencies {
    implementation("androidx.webkit:webkit:1.12.1")
}

val stageWeb = tasks.register<Copy>("stageWeb") {
    onlyIf { webDist.isDirectory }
    from(webDist)
    into(layout.buildDirectory.dir("staged-web/web"))
}

tasks.matching { it.name == "preBuild" }.configureEach {
    dependsOn(stageWeb)
}
