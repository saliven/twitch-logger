module "apps" {
  source = "./modules/apps"

  clickhouse = var.clickhouse
  api        = var.api
  ingest     = var.ingest

  docker_repository = var.docker_repository
  docker_registry   = var.docker_registry
  twitch_channels   = var.twitch_channels
  twitch_ignored    = var.twitch_ignored
}
