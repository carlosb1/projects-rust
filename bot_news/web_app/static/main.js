

 function new_comment(articleid, comment) {
	console.log('Adding a new comment: ' + comment);
	var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
         if (this.readyState == 4 && this.status == 200) {
             alert(this.responseText);
         }
    };
    xhttp.open("POST","/new_comment/"+articleid, true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send("Your JSON Data Here");
}

 function like(userid, articleid) {
	console.log('User id:'+userid+" article:"+articleid);
}

 function approve(userid, articleid) {
	console.log('approve!');
}

 function fake(userid, articleid)  {
	console.log('faked');
	var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
         if (this.readyState == 4 && this.status == 200) {
             alert(this.responseText);
         }
    };
    xhttp.open("POST","/fake/"+articleid, true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send("Your JSON Data Here");
}

