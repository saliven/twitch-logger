variable "kube_context" {
  description = "Kubernetes context to use"
  type        = string
}

variable "ingress_domain" {
  description = "Domain used for the environment"
  type        = string
}

variable "twitch_channels" {
  description = "List of twitch channels to monitor"
  type        = list(string)
  default     = []
}

variable "twitch_ignored" {
  description = "List of twitch usernames to ignore"
  type        = list(string)
  default     = []
}

variable "docker_repository" {
  description = "Docker repository to use"
  type        = string
}

variable "docker_registry" {
  description = "Docker registry data"
  type = object({
    server   = string
    username = string
    password = string
    email    = string
  })
}

variable "clickhouse" {
  description = "Clickhouse configuration"
  type = object({
    url      = string
    database = string
  })
}

variable "api" {
  description = "API configuration"
  type = object({
    clickhouse = object({
      user     = string
      password = string
    })
  })
}

variable "ingest" {
  description = "Ingest configuration"
  type = object({
    clickhouse = object({
      user     = string
      password = string
    })
  })
}
