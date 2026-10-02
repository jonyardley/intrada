# Sourced by the android-* recipes: the SDK and a JDK, from Android Studio when nothing is set.
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
if [ -z "${JAVA_HOME:-}" ] && [ -d "/Applications/Android Studio.app/Contents/jbr/Contents/Home" ]; then
    export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
fi
export PATH="$ANDROID_HOME/platform-tools:$PATH"
if [ ! -d android/generated/host ]; then
    echo "✗ no bindings in android/generated; run just android-gen" >&2
    exit 1
fi
