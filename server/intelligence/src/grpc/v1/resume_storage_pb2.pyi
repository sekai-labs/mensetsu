from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class GetResumeDownloadUrlRequest(_message.Message):
    __slots__ = ("workspace_id", "user_id", "resume_upload_id")
    WORKSPACE_ID_FIELD_NUMBER: _ClassVar[int]
    USER_ID_FIELD_NUMBER: _ClassVar[int]
    RESUME_UPLOAD_ID_FIELD_NUMBER: _ClassVar[int]
    workspace_id: str
    user_id: str
    resume_upload_id: str
    def __init__(self, workspace_id: _Optional[str] = ..., user_id: _Optional[str] = ..., resume_upload_id: _Optional[str] = ...) -> None: ...

class GetResumeDownloadUrlResponse(_message.Message):
    __slots__ = ("url", "filename", "size_bytes")
    URL_FIELD_NUMBER: _ClassVar[int]
    FILENAME_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    url: str
    filename: str
    size_bytes: int
    def __init__(self, url: _Optional[str] = ..., filename: _Optional[str] = ..., size_bytes: _Optional[int] = ...) -> None: ...
