import javax.inject.Inject

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.roborazzi)
    alias(libs.plugins.ktfmt)
}

val generated = rootProject.layout.projectDirectory.dir("generated")

android {
    namespace = "com.intrada.android"
    compileSdk = 36

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

    sourceSets {
        getByName("main") { jniLibs.directories.add(generated.dir("jniLibs").asFile.path) }
    }

    testOptions {
        unitTests {
            isIncludeAndroidResources = true
            all {
                it.systemProperty("jna.library.path", generated.dir("host").asFile.path)
                // Robolectric's SDK 36 sandbox reaches into FileDescriptor internals.
                it.jvmArgs("--add-exports=java.base/jdk.internal.access=ALL-UNNAMED")
            }
        }
    }
}

// crux's typegen writes a build.gradle.kts beside the sources, which the compiler would read as
// Kotlin, so only the .kt files are copied in.
abstract class SyncGeneratedKotlin : DefaultTask() {
    @get:InputDirectory abstract val source: DirectoryProperty

    @get:OutputDirectory abstract val output: DirectoryProperty

    @get:Inject abstract val fs: FileSystemOperations

    @TaskAction
    fun sync() {
        fs.sync {
            from(source) { include("**/*.kt") }
            into(output)
        }
    }
}

val syncGeneratedKotlin =
    tasks.register<SyncGeneratedKotlin>("syncGeneratedKotlin") {
        source.set(generated.dir("kotlin"))
    }

androidComponents {
    onVariants { variant ->
        variant.sources.kotlin?.addGeneratedSourceDirectory(
            syncGeneratedKotlin,
            SyncGeneratedKotlin::output,
        )
    }
}

ktfmt { kotlinLangStyle() }

dependencies {
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.foundation)
    implementation(libs.compose.ui)
    implementation(libs.activity.compose)
    implementation(libs.lifecycle.viewmodel)
    implementation(libs.coroutines.android)
    implementation("${libs.jna.get()}@aar")

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
}
