

 function new_comment(articleid, userid) {
    var comment = $.trim($("#textarea_comment_"+articleid).val());
	console.log('Adding a new comment: ' + comment);
	var xhttp = new XMLHttpRequest();

    xhttp.open("POST","/"+ articleid + "/new_comment", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(JSON.stringify({ "userid": userid, "comment": comment }));
}
function save_tags(articleid, userid) {
    console.log('Save tags User id:'+userid+" article:"+articleid);
   
    let tags = $("#inputtags_"+articleid).val().split(",");
    console.log(tags); 
    var xhttp = new XMLHttpRequest();
    xhttp.open("POST","/"+ articleid + "/save_tags", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(JSON.stringify({ "userid": userid, "tags": tags}));
    


}

 function like(articleid, userid) {
    console.log('Like User id:'+userid+" article:"+articleid);
    var xhttp = new XMLHttpRequest();
    xhttp.open("POST","/"+ articleid + "/like", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(JSON.stringify({ "userid": userid}));
}
 function approve(articleid, userid) {
    console.log('approve!');
    var xhttp = new XMLHttpRequest();
    xhttp.open("POST","/"+ articleid + "/approve", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(JSON.stringify({ "userid": userid}));
}

 function fake(articleid, userid)  {
	console.log('faked');
	var xhttp = new XMLHttpRequest();
    xhttp.open("POST","/"+ articleid + "/fake", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(JSON.stringify({ "userid": userid}));
}

