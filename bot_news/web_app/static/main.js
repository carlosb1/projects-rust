import axios from 'axios';

const API_URL = 'http://127.0.0.1:8000';

export function new_comment(comment) {
	console.log('Adding a new comment: ' + comment);
}

export function like(userid, articleid) {
	console.log('User id:'+userid+" article:"+articleid);
}

export function approve(userid, articleid) {
	console.log('approve!');
}

export function fake(userid, articleid)  {
	console.log('faked');
	axios.put('/fake',{
		'id_user': '1',
		'id_article': '2'
	}).then(function (response) {
		console.log(response);
	}).catch(function(error) {
		console.log(error);
	});
}

