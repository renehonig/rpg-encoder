"""Main entry point for the application."""
from src.auth.login import authenticate
from src.utils.config import load_config


def main():
    """Run the application."""
    config = load_config("config.toml")
    user = authenticate(config["username"], config["password"])
    if user:
        print(f"Welcome, {user.name}")
    else:
        print("Authentication failed")


def parse_args():
    """Parse command line arguments."""
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="config.toml")
    return parser.parse_args()
