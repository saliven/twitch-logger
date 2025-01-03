variable "twitch_channels" {
  type        = list(string)
  description = "List of twitch channels to monitor"
}

variable "twitch_ignored" {
  type        = list(string)
  description = "List of twitch usernames to ignore"
}

variable "docker_repository" {
  type        = string
  description = "Docker repository to use"
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
