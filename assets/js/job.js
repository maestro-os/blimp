function start_job(name, version) {
	// Starting job
	var xmlHttp = new XMLHttpRequest();
	xmlHttp.open("POST", "/dashboard/job/start?name=" + name + "&version=" + version, false);
	xmlHttp.send(null);
	var job = JSON.parse(xmlHttp.responseText);

	// Redirection to job's page
	window.location.replace("/dashboard/job/" + job["id"]);
}

function update_logs(job_id) {
	var xmlHttp = new XMLHttpRequest();
	xmlHttp.open("GET", "/dashboard/job/" + job_id + "/logs", false);
	xmlHttp.send(null);

	var textarea = document.getElementById("logs");
	textarea.innerHTML = xmlHttp.responseText;
}
