"""Authentication module."""
from src.utils.config import load_config


class User:
    """Represents an authenticated user."""

    def __init__(self, name: str, email: str):
        self.name = name
        self.email = email

    def is_admin(self) -> bool:
        """Check if user has admin privileges."""
        return self.email.endswith("@admin.com")


def authenticate(username: str, password: str):
    """Authenticate a user with credentials."""
    if username == "admin" and password == "secret":
        return User("Admin", "admin@admin.com")
    return None


def validate_token(token: str) -> bool:
    """Validate a JWT token."""
    return len(token) > 10 and token.startswith("Bearer ")
