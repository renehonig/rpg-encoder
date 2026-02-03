"""Data models for the application."""
from typing import List, Optional


class Task:
    """Represents a task item."""

    def __init__(self, title: str, description: str = ""):
        self.title = title
        self.description = description
        self.completed = False

    def complete(self):
        """Mark the task as completed."""
        self.completed = True

    def __repr__(self) -> str:
        status = "done" if self.completed else "pending"
        return f"Task({self.title!r}, {status})"


class TaskList:
    """A collection of tasks."""

    def __init__(self):
        self.tasks: List[Task] = []

    def add(self, task: Task):
        """Add a task to the list."""
        self.tasks.append(task)

    def find(self, title: str) -> Optional[Task]:
        """Find a task by title."""
        for task in self.tasks:
            if task.title == title:
                return task
        return None

    def pending(self) -> List[Task]:
        """Get all pending tasks."""
        return [t for t in self.tasks if not t.completed]
