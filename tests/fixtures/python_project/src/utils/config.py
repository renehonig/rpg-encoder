"""Configuration utilities."""
import os
from typing import Dict, Any


def load_config(path: str) -> Dict[str, Any]:
    """Load configuration from a TOML file."""
    with open(path) as f:
        return parse_toml(f.read())


def parse_toml(content: str) -> Dict[str, Any]:
    """Parse TOML content into a dictionary."""
    result = {}
    for line in content.strip().split("\n"):
        if "=" in line:
            key, val = line.split("=", 1)
            result[key.strip()] = val.strip().strip('"')
    return result


def get_env_var(name: str, default: str = "") -> str:
    """Get an environment variable with a default."""
    return os.environ.get(name, default)
