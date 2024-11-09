from fastapi import HTTPException, status, Security

API_KEYS = [
    "9d207bf0-10f5-4d8f-a479-22ff5aeff8d1",
    "f47d4a2c-24cf-4745-937e-620a5963c0b8",
    "b7061546-75e8-444b-a2c4-f19655d07eb8",
]

# def get_api_key(
#     api_key_query: str = Security(api_key_query),
#     api_key_header: str = Security(api_key_header),
# ) -> str:
#     """
#     Retrieve and validate an API key from the query parameters or HTTP header.
#
#     Args:
#         api_key_header: The API key passed in the HTTP header.
#
#     Returns:
#         The validated API key.
#
#     Raises:
#         HTTPException: If the API key is invalid or missing.
#     """
#     if api_key_header in API_KEYS:
#         return api_key_header
#
#     raise HTTPException(
#         status_code=status.HTTP_401_UNAUTHORIZED,
#         detail="Invalid or missing API Key",
#     )
