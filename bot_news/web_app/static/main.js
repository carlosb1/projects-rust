

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

function sanitizeHTMLEntities(str) {
    if (str && typeof str === 'string') {
        str = str.replace(/</g,"&lt;");
        str = str.replace(/>/g,"&gt;");
        str = str.replace(/&lt;em&gt;/g,"<em>");
        str = str.replace(/&lt;\/em&gt;/g,"<\/em>");
    }
    return str;
}


function search() {
    let baseUrl = "http://0.0.0.0:7700";
    let index = "valueindex";
    let theUrl = `${baseUrl}/indexes/${index}/search?q=${search.value}&attributesToHighlight=*`;
    let lastRequest = undefined;
    if (lastRequest) { lastRequest.abort() }
    lastRequest = new XMLHttpRequest();
    lastRequest.open("GET", theUrl, true);

    if (localStorage.getItem('apiKey')) {
      lastRequest.setRequestHeader("x-Meili-API-Key", localStorage.getItem('apiKey'));
    }
    lastRequest.onload = function (e) {
        if (lastRequest.readyState === 4 && lastRequest.status === 200) {
            let sanitizedResponseText = sanitizeHTMLEntities(lastRequest.responseText);
            let httpResults = JSON.parse(sanitizedResponseText);
            results.innerHTML = '';
            let ids = httpResults.hits.map(({hit}) => hit.id);
            var xhttp = new XMLHttpRequest();
            xhttp.open("POST","/search", true);
            xhttp.setRequestHeader("Content-type", "application/json");
            xhttp.send(JSON.stringify({"ids": ids}));
        } else {
            console.error(lastRequest.statusText);
        }
    };
    lastRequest.send(null);

}



function login() {
    console.log('Apply login');
    let username = $("#username").val();
    let password = $("#password").val();

    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            if (this.responseText != "null" ||  this.responseText) {
               $("#username").disabled = true;
               $("#password").disabled = true;
            }
        }
    };
    xhttp.open("POST","/login", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(JSON.stringify({ "username": username, "password": password}));

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

