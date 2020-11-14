from dataclasses import dataclass


class AliasException(Exception):
    pass


@dataclass
class JsonResponseException(Exception):
    message: str = "Bad request"
    status: int = 400
