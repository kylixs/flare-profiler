Vue.component('d3-flamegraph', {
    template: '#d3-flamegraph-template',
    data() {
        return { checked: false, title: 'Check me' }
    },
    methods: {
    }
});

var flameGraph = d3.flamegraph()
    .width(960)
    .cellHeight(18)
    .transitionDuration(750)
    .minFrameSize(5)
    .transitionEase(d3.easeCubic)
    .sort(false)
    //Example to sort in reverse order
    //.sort(function(a,b){ return d3.descending(a.name, b.name);})
    .title("")
    .onClick(onClick)
    .differential(false)
    .inverted(true)
    .selfValue(false);

// flameGraph.label(function(d) {
//   return d.data.name + ", " + d.data.value+"ms";
// });

// Example on how to use custom tooltips using d3-tip.
// var tip = d3.tip()
//   .direction("s")
//   .offset([8, 0])
//   .attr('class', 'd3-flame-graph-tip')
//   .html(function(d) { return "name: " + d.data.name + ", value: " + d.data.value; });

// flameGraph.tooltip(tip);

var details = document.getElementById("details");
flameGraph.setDetailsElement(details);

// Example on how to use custom labels
// var label = function(d) {
//  return "name: " + d.name + ", value: " + d.value;
// }
// flameGraph.label(label);

// Example of how to set fixed chart height
// flameGraph.height(540);

// d3.json("stacks.json", function(error, data) {
// 	if (error) return console.warn(error);
// 	d3.select("#chart")
// 			.datum(data)
// 			.call(flameGraph);
// });

function set_d3_flamegraph_data(data) {
    d3.selectAll("#chart > svg > *").remove();
    d3.select("#chart")
        .datum(data)
        .call(flameGraph);
}

function search() {
    var term = document.getElementById("term").value;
    flameGraph.search(term);
}

function clear() {
    document.getElementById('term').value = '';
    flameGraph.clear();
}

function resetZoom() {
    flameGraph.resetZoom();
}

function onClick(d) {
    console.info("Clicked on " + d.data.name);
}