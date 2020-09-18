

 function new_comment(articleid, userid, comment) {
	console.log('Adding a new comment: ' + comment);
	var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
         if (this.readyState == 4 && this.status == 200) {
             console.log(this.responseText);
         }
    };
    xhttp.open("POST","/"+ articleid + "/comment", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(JSON.stringify({ "userid": userid, "comment": comment }));
}

 function like(userid, articleid) {
    console.log('Like User id:'+userid+" article:"+articleid);
    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
         if (this.readyState == 4 && this.status == 200) {
             console.log(this.responseText);
         }
    };
    xhttp.open("POST","/"+ articleid + "/like", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(JSON.stringify({ "userid": userid}));
}
 function approve(userid, articleid) {
    console.log('approve!');
    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
         if (this.readyState == 4 && this.status == 200) {
             console.log(this.responseText);
         }
    };
    xhttp.open("POST","/"+ articleid + "/approve", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(JSON.stringify({ "userid": userid}));
}

 function fake(userid, articleid)  {
	console.log('faked');
	var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
         if (this.readyState == 4 && this.status == 200) {
             console.log(this.responseText);
         }
    };
    xhttp.open("POST","/"+ articleid + "/fake", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send();
}

