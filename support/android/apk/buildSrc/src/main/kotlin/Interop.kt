import org.gradle.api.GradleException
import org.gradle.api.Project
import java.io.File
import java.util.Locale
import java.util.Properties

private const val SERVO_NDK_VERSION = "28.2.13676358"

/*
Some functions are extensions to the Project class, as to allow access to its public members.
 */

fun Project.getTargetDir(debug: Boolean, arch: String): String {
    val basePath = project.rootDir.parentFile.parentFile.parentFile.absolutePath
    return basePath + "/target/" + getSubTargetDir(debug, arch)
}

fun Project.getNativeTargetDir(debug: Boolean, arch: String): String {
    val rustTarget = getRustTarget(arch)
    val base = repoRoot().resolve("target").resolve(rustTarget)

    System.getenv("SERVO_TARGET_DIR")
        ?.let(::File)
        ?.takeIf { it.isDirectory }
        ?.let { configured ->
            if (configured.hasServoShellLibrary()) {
                return configured.absolutePath
            }
            throw missingServoShellError(configured, rustTarget)
        }

    readLocalProperty("servo.target.dir")
        ?.let(::File)
        ?.takeIf { it.isDirectory && it.hasServoShellLibrary() }
        ?.let { return it.absolutePath }

    val profileCandidates = if (debug) {
        listOf("debug", "checked-release", "release")
    } else {
        listOf("release", "checked-release", "debug")
    }
    for (profile in profileCandidates) {
        val candidate = base.resolve(profile)
        if (candidate.hasServoShellLibrary()) {
            return candidate.absolutePath
        }
    }

    throw missingServoShellError(base, rustTarget, profileCandidates)
}

fun getSubTargetDir(debug: Boolean, arch: String): String {
    val buildTypeDirectory = System.getenv("SERVO_TARGET_DIR")
        ?.let { File(it).name }
        ?: if (debug) "debug" else "release"
    return getRustTarget(arch) + "/" + buildTypeDirectory
}

fun Project.getJniLibsPath(debug: Boolean, arch: String): String =
    getTargetDir(debug, arch) + "/jniLibs"

fun getRustTarget(arch: String): String {
    return when (arch.lowercase(Locale.getDefault())) {
        "armv7" -> "armv7-linux-androideabi"
        "arm64" -> "aarch64-linux-android"
        "x86" -> "i686-linux-android"
        "x64" -> "x86_64-linux-android"
        else -> throw GradleException("Invalid target architecture $arch")
    }
}

fun getNDKAbi(arch: String): String {
    return when (arch.lowercase(Locale.getDefault())) {
        "armv7" -> "armeabi-v7a"
        "arm64" -> "arm64-v8a"
        "x86" -> "x86"
        "x64" -> "x86_64"
        else -> throw GradleException("Invalid target architecture $arch")
    }
}

fun Project.getServoMinSdk(): Int {
    val propertiesFile = File(project.rootDir, "servo.properties")
    val properties = Properties()
    propertiesFile.inputStream().use { instr ->
        properties.load(instr)
    }
    val minSdk = properties.getProperty("android.minSdk")
        ?: throw GradleException("`android.minSdk` is missing from ${propertiesFile.absolutePath}")
    return minSdk.trim().toInt()
}

private fun Project.repoRoot(): File =
    project.rootDir.parentFile.parentFile.parentFile

private fun File.hasServoShellLibrary(): Boolean =
    resolve("libservoshell.so").isFile

private fun Project.readLocalProperty(key: String): String? {
    val localProperties = File(project.rootDir, "local.properties")
    if (!localProperties.exists()) return null
    val properties = Properties()
    localProperties.inputStream().use { instr ->
        properties.load(instr)
    }
    return normalizeAndroidPath(properties.getProperty(key))
}

private fun missingServoShellError(
    base: File,
    rustTarget: String,
    profileCandidates: List<String> = listOf("debug", "checked-release", "release"),
): GradleException {
    val checkedPaths = profileCandidates.joinToString("\n") { profile ->
        "  - ${base.resolve(profile).resolve("libservoshell.so").absolutePath}"
    }
    return GradleException(
        """
        libservoshell.so not found for $rustTarget.

        Android Gradle packaging needs a prior Servo mach build for the same architecture.

        Checked:
        $checkedPaths

        Build first (Linux/WSL2 or macOS):
          ./mach build --target $rustTarget --profile checked-release

        Or from repo root:
          .\scripts\build-android.ps1

        Optional overrides:
          SERVO_TARGET_DIR=<dir-with-libservoshell.so>
          servo.target.dir=<dir-with-libservoshell.so> in local.properties
        """.trimIndent()
    )
}

private fun normalizeAndroidPath(path: String?): String? {
    if (path.isNullOrBlank()) return null
    return path
        .trim()
        .replace("\\:", ":")
        .replace("\\\\", "\\")
}

private fun readNdkRevision(dir: File): String? {
    val sourceProperties = dir.resolve("source.properties")
    if (!sourceProperties.isFile) return null
    val properties = Properties()
    sourceProperties.inputStream().use { instr ->
        properties.load(instr)
    }
    return properties.getProperty("Pkg.Revision")?.trim()
}

private fun isServoNdk(dir: File?): Boolean {
    if (dir == null || !dir.isDirectory) return false
    val revision = readNdkRevision(dir)
    return revision == SERVO_NDK_VERSION || revision?.startsWith("28.") == true
}

private fun findServoNdkInSdk(sdkRoot: File?): File? {
    if (sdkRoot == null || !sdkRoot.isDirectory) return null
    val sideBySide = sdkRoot.resolve("ndk").resolve(SERVO_NDK_VERSION)
    if (isServoNdk(sideBySide)) {
        return sideBySide
    }
    val legacy = sdkRoot.resolve("ndk-bundle")
    if (isServoNdk(legacy)) {
        return legacy
    }
    return null
}

fun Project.getNdkDir(): String {
    val rootDir = project.rootDir
    var sdkRoot: File? = null

    // Read environment variable used in rust build system
    var ndkDir = normalizeAndroidPath(System.getenv("ANDROID_NDK_ROOT"))?.let(::File)
    if (!isServoNdk(ndkDir)) {
        ndkDir = null
    }

    val localProperties = File(rootDir, "local.properties")
    if (localProperties.exists()) {
        val properties = Properties()
        localProperties.inputStream().use { instr ->
            properties.load(instr)
        }

        sdkRoot = normalizeAndroidPath(properties.getProperty("sdk.dir"))?.let(::File)
        if (ndkDir == null) {
            ndkDir = normalizeAndroidPath(properties.getProperty("ndk.dir"))?.let(::File)
                ?.takeIf(::isServoNdk)
        }
    }

    if (sdkRoot == null) {
        sdkRoot = sequenceOf(
            System.getenv("ANDROID_SDK_ROOT"),
            System.getenv("ANDROID_HOME"),
            System.getenv("LOCALAPPDATA")?.let { "$it\\Android\\Sdk" }
        )
            .mapNotNull(::normalizeAndroidPath)
            .map(::File)
            .firstOrNull { it.isDirectory }
    }

    if (ndkDir == null) {
        ndkDir = findServoNdkInSdk(sdkRoot)
    }

    if (!isServoNdk(ndkDir)) {
        throw GradleException(
            "Servo requires Android NDK r28 ($SERVO_NDK_VERSION). " +
                "Set ANDROID_NDK_ROOT, set ndk.dir in local.properties, " +
                "or install that NDK version in Android SDK Manager."
        )
    }
    return ndkDir!!.absolutePath
}

fun Project.getNdkBuildPath(): String {
    val ndkDir = File(getNdkDir())
    val executableName =
        if (System.getProperty("os.name").lowercase(Locale.getDefault()).contains("windows")) {
            "ndk-build.cmd"
        } else {
            "ndk-build"
        }
    val executable = ndkDir.resolve(executableName)
    if (!executable.exists()) {
        throw GradleException("ndk-build not found at ${executable.absolutePath}")
    }
    return executable.absolutePath
}
