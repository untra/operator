terraform {
  required_version = ">= 1.2"

  required_providers {
    coder = {
      source  = "coder/coder"
      version = ">= 2.5"
    }
  }
}

variable "agent_id" {
  type        = string
  description = "The ID of a Coder agent."
}

variable "port" {
  type        = number
  description = "The port for the operator REST API server."
  default     = 7008
}

variable "display_name" {
  type        = string
  description = "The display name for the operator application in the Coder dashboard."
  default     = "Operator"
}

variable "slug" {
  type        = string
  description = "The slug for the operator application."
  default     = "operator"
}

variable "install_version" {
  type        = string
  description = "The version of operator to install (must match a GitHub release tag)."
  default     = "0.2.0"
}

variable "install_prefix" {
  type        = string
  description = "The directory to install the operator binary into."
  default     = "/tmp/operator"
}

variable "log_path" {
  type        = string
  description = "The path to write operator log output."
  default     = "/tmp/operator.log"
}

variable "config_toml" {
  type        = string
  description = "Raw TOML configuration content. When set, this is written verbatim to .tickets/operator/config.toml instead of the auto-generated config."
  default     = ""
}

variable "max_parallel_agents" {
  type        = number
  description = "Maximum number of parallel agents operator can run."
  default     = 2
}

variable "session_wrapper" {
  type        = string
  description = "Session wrapper type for agent terminal sessions."
  default     = "tmux"
  validation {
    condition     = contains(["tmux", "cmux", "zellij"], var.session_wrapper)
    error_message = "session_wrapper must be one of: tmux, cmux, zellij."
  }
}

variable "share" {
  type    = string
  default = "owner"
  validation {
    condition     = contains(["owner", "authenticated", "public"], var.share)
    error_message = "share must be one of: owner, authenticated, public."
  }
}

variable "order" {
  type        = number
  description = "The order determines the position of the app in the Coder dashboard. Lower values appear first."
  default     = null
}

variable "group" {
  type        = string
  description = "The name of a group that this app belongs to."
  default     = null
}

variable "offline" {
  type        = bool
  description = "Skip downloading operator from GitHub. Requires a pre-installed binary at install_prefix."
  default     = false
}

variable "use_cached" {
  type        = bool
  description = "Use a cached operator binary if present, otherwise download from GitHub."
  default     = false
}

resource "coder_script" "operator" {
  agent_id     = var.agent_id
  display_name = "Operator"
  icon         = "/icon/terminal.svg"
  script = templatefile("${path.module}/run.sh", {
    VERSION         = var.install_version,
    PORT            = var.port,
    INSTALL_PREFIX  = var.install_prefix,
    LOG_PATH        = var.log_path,
    CONFIG_TOML     = var.config_toml,
    MAX_PARALLEL    = var.max_parallel_agents,
    SESSION_WRAPPER = var.session_wrapper,
    OFFLINE         = var.offline,
    USE_CACHED      = var.use_cached,
  })
  run_on_start = true

  lifecycle {
    precondition {
      condition     = !var.offline || !var.use_cached
      error_message = "offline and use_cached cannot both be true."
    }
  }
}

resource "coder_app" "operator" {
  agent_id     = var.agent_id
  slug         = var.slug
  display_name = var.display_name
  url          = "http://localhost:${var.port}"
  icon         = "/icon/terminal.svg"
  subdomain    = false
  share        = var.share
  order        = var.order
  group        = var.group

  healthcheck {
    url       = "http://localhost:${var.port}/api/v1/health"
    interval  = 5
    threshold = 6
  }
}
