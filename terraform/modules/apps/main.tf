resource "kubernetes_namespace" "logging" {
  metadata {
    name = "logging"
  }
}

# Configuration for docker registry
resource "kubernetes_secret" "image_pull_secret" {
  metadata {
    name      = "image-pull-secret"
    namespace = kubernetes_namespace.logging.metadata[0].name
    labels = {
      app = "logging"
    }
  }

  type = "kubernetes.io/dockerconfigjson"

  data = {
    ".dockerconfigjson" = jsonencode({
      auths = {
        "${var.docker_registry.server}" = {
          "username" = var.docker_registry.username
          "password" = var.docker_registry.password
          "email"    = var.docker_registry.email
          "auth"     = base64encode("${var.docker_registry.username}:${var.docker_registry.password}")
        }
      }
    })
  }
}

# Configuration for both applications
resource "kubernetes_config_map" "logging_config" {
  metadata {
    name      = "logging-config"
    namespace = kubernetes_namespace.logging.metadata[0].name
    labels = {
      app = "logging"
    }
  }

  data = {
    CLICKHOUSE_URL      = var.clickhouse.url
    CLICKHOUSE_DATABASE = var.clickhouse.database
  }
}

# Configuration for logging ingest
resource "kubernetes_secret" "logging_ingest_secret" {
  metadata {
    name      = "logging-ingest-secret"
    namespace = kubernetes_namespace.logging.metadata[0].name
    labels = {
      app = "logging-ingest"
    }
  }

  type = "Opaque"

  data = {
    CLICKHOUSE_USER     = var.api.clickhouse.user
    CLICKHOUSE_PASSWORD = var.api.clickhouse.password
  }
}

resource "kubernetes_config_map" "logging_ingest_config" {
  metadata {
    name      = "logging-ingest-config"
    namespace = kubernetes_namespace.logging.metadata[0].name
    labels = {
      app = "logging-ingest"
    }
  }

  data = {
    TWITCH_CHANNELS = "[${join(",", var.twitch_channels)}]"
    TWITCH_IGNORED  = "[${join(",", var.twitch_ignored)}]"
  }
}

resource "kubernetes_service_account" "loggin_ingest_service_account" {
  metadata {
    name      = "logging-ingest-service-account"
    namespace = kubernetes_namespace.logging.metadata[0].name
    labels = {
      app = "logging-ingest"
    }
  }

  image_pull_secret {
    name = kubernetes_secret.image_pull_secret.metadata[0].name
  }
}

resource "kubernetes_deployment" "logging_ingest" {
  metadata {
    name      = "logging-ingest"
    namespace = kubernetes_namespace.logging.metadata[0].name
    labels = {
      app = "logging-ingest"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "logging-ingest"
      }
    }

    template {
      metadata {
        name      = "logging-ingest"
        namespace = kubernetes_namespace.logging.metadata[0].name
        labels = {
          app = "logging-ingest"
        }
        annotations = {
          "prometheus.io/scrape" = "true"
          "prometheus.io/port"   = "8080"
          "prometheus.io/path"   = "/metrics"
        }
      }

      spec {
        service_account_name = kubernetes_service_account.loggin_ingest_service_account.metadata[0].name

        container {
          name              = "logging-ingest"
          image             = "${var.docker_repository}/logging-ingest:latest"
          image_pull_policy = "Always"

          env {
            name  = "PORT"
            value = "8080"
          }

          env {
            name  = "RUST_ENV"
            value = "production"
          }

          env_from {
            config_map_ref {
              name = kubernetes_config_map.logging_config.metadata[0].name
            }
          }

          env_from {
            config_map_ref {
              name = kubernetes_config_map.logging_ingest_config.metadata[0].name
            }
          }

          env_from {
            secret_ref {
              name = kubernetes_secret.logging_ingest_secret.metadata[0].name
            }
          }

          resources {
            limits = {
              cpu    = "0.5"
              memory = "512Mi"
            }
            requests = {
              cpu    = "0.1"
              memory = "128Mi"
            }
          }

          liveness_probe {
            http_get {
              path = "/health"
              port = "8080"
            }
            initial_delay_seconds = 2
            period_seconds        = 5
          }

          readiness_probe {
            http_get {
              path = "/health"
              port = "8080"
            }
            initial_delay_seconds = 2
            period_seconds        = 5
          }
        }
      }
    }
  }
}
