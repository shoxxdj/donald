var express=require('express');
var app=express();

var bodyParser = require('body-parser')
app.use(bodyParser.text({type:"*/*"}));

app.get('/',(req,res)=>{
 res.end("ok");
});

app.post('/donald/whitehouse',(req,res)=>{
 console.log(req.body);
 res.end('ok');
 console.log("*".repeat(50));
});

app.listen(8080);
