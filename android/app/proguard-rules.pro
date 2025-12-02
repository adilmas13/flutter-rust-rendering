# Flutter specific rules
-keep class io.flutter.** { *; }
-keep class io.flutter.plugins.** { *; }

# Keep JNI methods
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep the GameNative class for JNI
-keep class com.example.flutter_con.GameNative { *; }

# Ignore missing Play Core classes (not used in this app)
-dontwarn com.google.android.play.core.**
