use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sdk {
    pub candidate: String,
    pub name: String,
    pub description: String,
    pub website: Option<String>,
    pub latest_version: Option<String>,
    pub installed_version: Option<String>,
    pub installed: bool,
    pub category: Category,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Category {
    #[serde(rename = "LANGUAGES")]
    Languages,
    #[serde(rename = "BUILD_TOOLS")]
    BuildTools,
    #[serde(rename = "FRAMEWORKS")]
    Frameworks,
    #[serde(rename = "SERVERS")]
    Servers,
    #[serde(rename = "MQ")]
    Mq,
    #[serde(rename = "TOOLS")]
    Tools,
    #[serde(rename = "OTHER")]
    Other,
}

impl Category {
    pub fn from_name(candidate: &str, description: &str) -> Self {
        let normalized = candidate.to_lowercase();

        // Languages
        if matches!(normalized.as_str(),
            "java" | "kotlin" | "scala" | "groovy" | "clojure" | "jruby" | "jython" |
            "ceylon" | "ballerina" | "crash" | "sbl"
        ) {
            return Category::Languages;
        }

        // Build Tools
        if matches!(normalized.as_str(),
            "maven" | "mvnd" | "gradle" | "sbt" | "ant" | "bld"
        ) {
            return Category::BuildTools;
        }

        // Frameworks
        if matches!(normalized.as_str(),
            "springboot" | "micronaut" | "quarkus" | "vertx"
        ) {
            return Category::Frameworks;
        }

        // Servers
        if matches!(normalized.as_str(),
            "tomcat" | "jetty" | "payara" | "wildfly" | "tomee" | "liberty" |
            "glowroot" | "tackle" | "akka"
        ) {
            return Category::Servers;
        }

        // MQ
        if matches!(normalized.as_str(), "activemq" | "kcctl") {
            return Category::Mq;
        }

        // Tools
        if matches!(normalized.as_str(),
            "visualvm" | "jmc" | "jmeter" | "gatling" | "selenide" | "cucumber" |
            "testng" | "mockito" | "asciidoctorj" | "jbang" | "lombok" | "cdi" |
            "jakartaee" | "leiningen" | "helidon" | "apache" | "zookeeper" |
            "consul" | "etcd" | "redis"
        ) {
            return Category::Tools;
        }

        // Description-based matching
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("programming language") || desc_lower.contains("jvm language")
            || desc_lower.contains("language") || desc_lower.contains("compiler") {
            return Category::Languages;
        }

        if desc_lower.contains("build tool") || desc_lower.contains("build automation")
            || desc_lower.contains("project management") {
            return Category::BuildTools;
        }

        if desc_lower.contains("framework") || desc_lower.contains("microframework")
            || desc_lower.contains("application framework") {
            return Category::Frameworks;
        }

        if desc_lower.contains("server") || desc_lower.contains("application server")
            || desc_lower.contains("servlet container") || desc_lower.contains("web server") {
            return Category::Servers;
        }

        if desc_lower.contains("tool") || desc_lower.contains("utility")
            || desc_lower.contains("monitoring") || desc_lower.contains("testing")
            || desc_lower.contains("debugging") || desc_lower.contains("documentation") {
            return Category::Tools;
        }

        // Name pattern matching as last resort
        if normalized.contains("lang") || normalized.contains("language") {
            return Category::Languages;
        }
        if normalized.contains("build") || normalized.contains("gradle")
            || normalized.contains("maven") || normalized.contains("ant") {
            return Category::BuildTools;
        }
        if normalized.contains("framework") || normalized.contains("spring")
            || normalized.contains("micro") || normalized.contains("quarkus") {
            return Category::Frameworks;
        }
        if normalized.contains("server") || normalized.contains("tomcat")
            || normalized.contains("jetty") || normalized.contains("wildfly") {
            return Category::Servers;
        }

        Category::Other
    }
}
