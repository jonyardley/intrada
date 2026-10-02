plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.roborazzi)
    alias(libs.plugins.ktfmt)
    alias(libs.plugins.detekt)
}

val generated = rootProject.layout.projectDirectory.dir("generated")

android {
    namespace = "com.intrada.android"
    compileSdk = 36
    ndkVersion = providers.gradleProperty("intrada.ndkVersion").get()

    defaultConfig {
        applicationId = "com.intrada.android"
        minSdk = 28
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
        // The bridge is built for these two only (just android-package).
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures { compose = true }

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
    implementation(libs.coroutines.android)
    implementation(project(":bridge"))

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
