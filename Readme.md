curl -s \
-X POST \
--user "b61dd8d26056458a5226a7667a2199ab:a29f07858b7cec12f19df5f20df40e02" \
https://api.mailjet.com/v3.1/send \
-H 'Content-Type: application/json' \
-d '{ "Messages":[ 
    { "From": 
        { "Email": "luis.manuel.neto@proton.me", "Name": "Mailjet Pilot" }, 
          "To": [ { "Email": "lmpneto137@gmail.com", "Name": "passenger 1" } ],
          "Subject": "Your email flight plan!",
          "TextPart": "Dear passenger 1, welcome to Mailjet! May the delivery force be with you!",
          "HTMLPart": "<h3>Dear passenger 1, welcome to <a href=\"https://www.mailjet.com/\">Mailjet</a>!</h3><br />May the delivery force be with you!" } ]
    }'
