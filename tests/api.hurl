# Sanity check
GET {{baseurl}}
[Options]
delay: 1s
HTTP 200

# Unaunthicated transactions
POST {{baseurl}}/login
{
    "email": "foo@example.com",
    "password": "bar"
}
HTTP 401

POST {{baseurl}}/register
{
    "did": 1234
}
HTTP 401

POST {{baseurl}}/cmd
{
    "did": 1234,
    "cmd": "Lorem ipsum dolor sit amet"
}
HTTP 401

POST {{baseurl}}/upload
{
    "did": 1234,
    "data": "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod\ntempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam,\nquis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo\nconsequat. Duis aute irure dolor in reprehenderit in voluptate velit esse\ncillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non\nproident, sunt in culpa qui officia deserunt mollit anim id est laborum."
}
HTTP 401

POST {{baseurl}}/download
{
    "start": 0,
    "end": 4294967295
}
HTTP 401

GET {{baseurl}}/devices
HTTP 401

# Login

POST {{baseurl}}/signup
{
    "email": "foo@example.com",
    "password": "bar"
}
HTTP 200

POST {{baseurl}}/login
{
    "email": "foo@example.com",
    "password": "bar"
}
HTTP 200

POST {{baseurl}}/register
{
    "did": 1234
}
HTTP 200

# Upload

POST {{baseurl}}/upload
{
    "did": 1234,
    "data": "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod\ntempor incididunt ut labore et dolore magna aliqua."
}
HTTP 200
""

#  Unregistered

POST {{baseurl}}/upload
{
    "did": 5678,
    "data": "Duis aute irure dolor in reprehenderit in voluptate velit esse\ncillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non\nproident, sunt in culpa qui officia deserunt mollit anim id est laborum."
}
HTTP 200

# Commands

POST {{baseurl}}/cmd
{
    "did": 1234,
    "cmd": "Lorem ipsum dolor sit amet"
}
HTTP 200

POST {{baseurl}}/upload
{
    "did": 1234,
    "data": "Ut enim ad minim veniam,\nquis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo\nconsequat."
}
HTTP 200
"Lorem ipsum dolor sit amet"

# Downloads

POST {{baseurl}}/download
[Options]
output: -
{
    "start": 0,
    "end": 4294967295
}
HTTP 200
[Asserts]
jsonpath "$" count == 3
jsonpath "$[0].did" == 1234
jsonpath "$[0].data" == "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod\ntempor incididunt ut labore et dolore magna aliqua."
jsonpath "$[1].did" == 5678
jsonpath "$[1].data" == "Duis aute irure dolor in reprehenderit in voluptate velit esse\ncillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non\nproident, sunt in culpa qui officia deserunt mollit anim id est laborum."
jsonpath "$[2].did" == 1234
jsonpath "$[2].data" == "Ut enim ad minim veniam,\nquis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo\nconsequat."

# List devices

GET {{baseurl}}/devices
[Options]
output: -
HTTP 200
[Asserts]
jsonpath "$" count == 1
jsonpath "$[0]" == 1234

# Logging out

GET {{baseurl}}/logout
HTTP 200

POST {{baseurl}}/register
{
    "did": 1234
}
HTTP 401

