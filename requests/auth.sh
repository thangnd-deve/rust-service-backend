# register
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "thangnd.deve",
    "email": "thangnd.deve@gmail.com",
    "password": "123"
  }'

# login
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "thangnd.deve@gmail.com",
    "password": "123"
}'


# verify token is valid or not.
curl -X POST http://localhost:8000/auth/verify-token \
-H "Content-Type: application/json" \
-d '{
    "token": "{token}"
}'
