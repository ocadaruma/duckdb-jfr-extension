import org.gradle.api.tasks.testing.logging.TestExceptionFormat
import org.gradle.api.tasks.testing.logging.TestLogEvent

plugins {
    id("java-library")
    id("maven-publish")
    id("signing")
}

repositories {
    mavenCentral()
}

group = "com.mayreh.duckdb"
version = "$version" + if ((property("snapshot") as String).toBoolean()) "-SNAPSHOT" else ""
extra["isReleaseVersion"] = !version.toString().endsWith("SNAPSHOT")

java {
    sourceCompatibility = JavaVersion.VERSION_1_8
    targetCompatibility = JavaVersion.VERSION_1_8
    withJavadocJar()
    withSourcesJar()
}

tasks.processResources {
    from(file("$projectDir/../build"))
}

dependencies {
    runtimeOnly("org.duckdb:duckdb_jdbc:0.9.1")
    testImplementation(platform("org.junit:junit-bom:5.9.1"))
    testImplementation("org.junit.jupiter:junit-jupiter")
}

tasks.test {
    useJUnitPlatform()

    classpath += files("$projectDir/../test-data")

    testLogging {
        events(TestLogEvent.FAILED,
                TestLogEvent.PASSED,
                TestLogEvent.SKIPPED,
                TestLogEvent.STANDARD_OUT)
        exceptionFormat = TestExceptionFormat.FULL
        showExceptions = true
        showCauses = true
        showStackTraces = true
        showStandardStreams = true
    }
}

publishing {
    repositories {
        maven {
            url = if (project.extra["isReleaseVersion"] as Boolean) {
                uri("https://oss.sonatype.org/service/local/staging/deploy/maven2/")
            } else {
                uri("https://oss.sonatype.org/content/repositories/snapshots/")
            }
            credentials {
                username = findProperty("sonatypeUsername").toString()
                password = findProperty("sonatypePassword").toString()
            }
        }
    }

    publications {
        create<MavenPublication>("mavenJava") {
            from(components["java"])
            artifactId = project.name
            pom {
                name = "DuckDB JFR JDBC"
                description = "JFR support for DuckDB JDBC Driver"
                url = "https://github.com/ocadaruma/duckdb-jfr-extension"

                scm {
                    connection = "scm:git:git@github.com:ocadaruma/duckdb-jfr-extension.git"
                    url = "git@github.com:ocadaruma/duckdb-jfr-extension.git"
                }
                licenses {
                    license {
                        name = "The Apache License, Version 2.0"
                        url = "https://raw.githubusercontent.com/ocadaruma/duckdb-jfr-extension/master/LICENSE"
                    }
                }
                developers {
                    developer {
                        id = "ocadaruma"
                        name = "Haruki Okada"
                    }
                }
            }
        }
    }
}

signing {
    isRequired = (project.extra["isReleaseVersion"] as Boolean) && gradle.taskGraph.hasTask("publish")
    sign(publishing.publications["mavenJava"])
}
