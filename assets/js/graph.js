var canvas = document.getElementById('graph');
var ctx = canvas.getContext('2d');
ctx.fillStyle = '#dfdfdf';

// Making call to get the graph
var xmlHttp = new XMLHttpRequest();
xmlHttp.open("GET", "/dependency_graph", false);
xmlHttp.send(null);
var graph = JSON.parse(xmlHttp.responseText);

// TODO Draw nodes and edges
ctx.fillRect(100, 100, 10, 10);

ctx.font = "30px Open Sans, sans serif";
ctx.fillText("Dependency graph", 10, 10);
