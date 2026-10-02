import javax.inject.Inject

plugins { alias(libs.plugins.android.library) }

val generated = rootProject.layout.projectDirectory.dir("generated")

// The generated Kotlin lives in its own module so the app's warnings-as-errors gate covers only
// hand-written code: crux's typegen and UniFFI emit warnings we cannot fix at the source (#2263).
android {
    namespace = "com.intrada.bridge"
    compileSdk = 36

    defaultConfig { minSdk = 28 }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets {
        getByName("main") { jniLibs.directories.add(generated.dir("jniLibs").asFile.path) }
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

dependencies { api("${libs.jna.get()}@aar") }
