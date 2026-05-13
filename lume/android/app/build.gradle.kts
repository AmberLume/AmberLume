plugins {
    alias(libs.plugins.android.application)
}

val workspaceRoot: File = rootProject.projectDir.parentFile.parentFile

android {
    namespace = "com.vastausf.lume"
    compileSdk {
        version = release(36) {
            minorApiLevel = 1
        }
    }
    ndkVersion = "30.0.14904198"

    defaultConfig {
        applicationId = "com.vastausf.lume"
        minSdk = 31
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"

        ndk {
            abiFilters += listOf("arm64-v8a")
        }
    }

    buildTypes {
        debug {
            isDebuggable = true
            isJniDebuggable = true
        }
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    sourceSets {
        getByName("main") {
            assets.directories.add("$workspaceRoot/target/distribution/assets")
        }
    }

    androidResources {
        noCompress += listOf("alpaca", "manifest")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    implementation(libs.androidx.games.activity)
    implementation(libs.androidx.appcompat)
}

// --- Rust build integration ---

fun rustTask(name: String, release: Boolean) = tasks.register<Exec>(name) {
    workingDir = workspaceRoot
    val args = mutableListOf(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "--platform", "31",
        "-o", "${projectDir}/src/main/jniLibs",
        "build", "-p", "android"
    )
    if (release) args += "--release"
    commandLine(args)

    inputs.dir(File(workspaceRoot, "amber_lume"))
    inputs.dir(File(workspaceRoot, "lume/core"))
    inputs.dir(File(workspaceRoot, "lume/android/src"))
    inputs.file(File(workspaceRoot, "Cargo.toml"))
    inputs.file(File(workspaceRoot, "lume/android/Cargo.toml"))
    outputs.dir(File(projectDir, "src/main/jniLibs"))
}

val rustBuildDebug = rustTask("rustBuildDebug", release = false)
val rustBuildRelease = rustTask("rustBuildRelease", release = true)

tasks.matching { it.name == "mergeDebugJniLibFolders" }.configureEach {
    dependsOn(rustBuildDebug)
}
tasks.matching { it.name == "mergeReleaseJniLibFolders" }.configureEach {
    dependsOn(rustBuildRelease)
}
