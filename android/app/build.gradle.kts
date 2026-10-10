plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.roborazzi)
    alias(libs.plugins.ktfmt)
    alias(libs.plugins.detekt)
}

val generated = rootProject.layout.projectDirectory.dir("generated")
val iosProject = rootProject.layout.projectDirectory.file("../ios/project.yml")

android {
    namespace = "com.intrada.android"
    compileSdk = 37
    ndkVersion = providers.gradleProperty("intrada.ndkVersion").get()

    defaultConfig {
        applicationId = "com.intrada.android"
        minSdk = 28
        targetSdk = 36
        // Name: the tag, else the iPhone app's, so no build reports to an uncut release (#1961).
        versionCode = providers.environmentVariable("ANDROID_VERSION_CODE").orNull?.toInt() ?: 1
        versionName =
            providers
                .environmentVariable("ANDROID_VERSION_NAME")
                .orElse(
                    providers.fileContents(iosProject).asText.map {
                        Regex("""MARKETING_VERSION: "([0-9.]+)"""").find(it)?.groupValues?.get(1)
                            ?: error("no MARKETING_VERSION in ios/project.yml")
                    }
                )
                .get()
        // The bridge is built for these two only (just android-package).
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
        // Unset or not https keeps Sentry off, which is how CI and local builds run (#2497).
        val sentryDsn =
            providers
                .environmentVariable("SENTRY_DSN_ANDROID")
                .orElse(providers.gradleProperty("SENTRY_DSN_ANDROID"))
                .getOrElse("")
                .trim()
        buildConfigField("String", "SENTRY_DSN", "\"$sentryDsn\"")
    }

    // The upload key, set only in release-play.yml; Play App Signing holds the app key (#2499).
    val uploadKeystore = providers.environmentVariable("ANDROID_UPLOAD_KEYSTORE").orNull
    if (uploadKeystore != null) {
        signingConfigs {
            create("upload") {
                storeFile = file(uploadKeystore)
                storePassword =
                    providers.environmentVariable("ANDROID_UPLOAD_KEYSTORE_PASSWORD").get()
                keyAlias = providers.environmentVariable("ANDROID_UPLOAD_KEY_ALIAS").get()
                keyPassword = providers.environmentVariable("ANDROID_UPLOAD_KEY_PASSWORD").get()
            }
        }
        buildTypes { getByName("release") { signingConfig = signingConfigs.getByName("upload") } }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    lint {
        warningsAsErrors = true
        abortOnError = true
        checkTestSources = true
        // These turn red when Google ships a release, not on a change; bumps are their own PRs.
        disable +=
            setOf(
                "AndroidGradlePluginVersion",
                "GradleDependency",
                "NewerVersionAvailable",
                "OldTargetApi",
            )
    }

    testOptions {
        unitTests {
            isIncludeAndroidResources = true
            all {
                it.systemProperty("jna.library.path", generated.dir("host").asFile.path)
                val iosTheme = rootProject.file("../ios/Intrada/DesignSystem/Theme.swift")
                it.systemProperty("intrada.iosTheme", iosTheme.path)
                it.inputs
                    .file(iosTheme)
                    .withPropertyName("iosTheme")
                    .withPathSensitivity(PathSensitivity.NONE)
                it.inputs
                    .dir(layout.projectDirectory.dir("src/test/snapshots"))
                    .withPropertyName("snapshotReferences")
                    .withPathSensitivity(PathSensitivity.RELATIVE)
                it.inputs
                    .dir(generated.dir("host"))
                    .withPropertyName("bridgeHostLibrary")
                    .withPathSensitivity(PathSensitivity.NONE)
                // Robolectric's SDK 36 sandbox reaches into FileDescriptor internals.
                it.jvmArgs("--add-exports=java.base/jdk.internal.access=ALL-UNNAMED")
            }
        }
    }
}

kotlin { compilerOptions { allWarningsAsErrors = true } }

ktfmt { kotlinLangStyle() }

detekt {
    buildUponDefaultConfig = true
    config.setFrom(rootProject.file("config/detekt.yml"))
}

dependencies {
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.foundation)
    implementation(libs.compose.ui)
    implementation(libs.activity.compose)
    implementation(libs.lifecycle.viewmodel)
    implementation(libs.navigation.compose)
    implementation(libs.coroutines.android)
    implementation(project(":bridge"))
    implementation(libs.sentry.android)
    implementation(libs.coroutines.play.services)
    implementation(libs.mlkit.text.recognition)
    implementation(libs.mlkit.document.scanner)

    testImplementation(libs.jna)
    testImplementation(libs.junit)
    testImplementation(libs.coroutines.test)
    testImplementation(libs.robolectric)
    testImplementation(libs.roborazzi)
    testImplementation(libs.roborazzi.compose)
    testImplementation(libs.roborazzi.junit.rule)
    testImplementation(platform(libs.compose.bom))
    testImplementation(libs.compose.ui.test.junit4)
    debugImplementation(libs.compose.ui.test.manifest)

    detektPlugins(libs.compose.rules.detekt)
}
