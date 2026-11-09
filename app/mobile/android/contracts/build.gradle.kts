plugins {
    id("org.jetbrains.kotlin.jvm")
}

kotlin {
    sourceSets.named("main") {
        kotlin.srcDir("../../../../contracts/native/kotlin/src/main/kotlin")
    }
}

dependencies {
    implementation("org.jetbrains.kotlin:kotlin-reflect:1.9.23")
    implementation("com.squareup.moshi:moshi-kotlin:1.15.1")
    implementation("com.squareup.okhttp3:okhttp:4.12.0")
}
