import * as d3 from 'd3/build/d3';

export default {
    getD3LineChart: function (elementId, data){
        var svg = d3.select("#"+elementId).select("svg"),
            margin = {top: 20, right: 20, bottom: 110, left: 40},
            width = +svg.attr("width") - margin.left - margin.right,
            height = +svg.attr("height") - margin.top - margin.bottom;

        var xData = [];

        for (let i = 0; i < 30000; i++) {
            xData.push(i);
        }

        var x = d3.scaleLinear().range([0, width]);
        var x2 = d3.scaleLinear().range([0, width]);
        var y = d3.scaleLinear().range([height, 0]);

        var xAxis = d3.axisBottom(x).ticks(30);


        x.domain(d3.extent(data, function(d) {
            return d.date;
        }));
        y.domain([0, d3.max(data, function(d) { return d.price; })]);
        x2.domain(x.domain());

        var zoom = d3.zoom()
            .scaleExtent([1/20, 50])
            .translateExtent([[0, 0], [width, height]])
            .extent([[0, 0], [width, height]])
            .on("zoom", zoomed);

        var area = d3.area()
            .curve(d3.curveMonotoneX)
            .x(function(d) { return x(d.date); })
            .y0(height)
            .y1(function(d) { return y(d.price); });

        svg.append("defs").append("clipPath")
            .attr("id", "clip")
            .append("rect")
            .attr("width", width)
            .attr("height", height);


        svg.append("rect")
            .attr("class", "zoom")
            .attr("width", width)
            .attr("height", height)
            .attr("transform", "translate(" + margin.left + "," + margin.top + ")")

        var focus = svg.append("g")
            .attr("class", "focus")
            .attr("transform", "translate(" + margin.left + "," + margin.top + ")");


        focus.append("path")
            .datum(data)
            .attr("class", "area")
            .attr("d", area);

        focus.append("g")
            .attr("class", "axis axis--x")
            .attr("transform", "translate(0," + height + ")")
            .call(xAxis);

        var tooltip = d3.select("#d3Div")
            .append("div")
            .attr("class","tooltip")
            .style("opacity",0.0);

        var tooltipText = tooltip.append("div")
            .attr("class", "desText");

        focus.on("mouseover", function (d,i) {
            let numAry = d3.mouse(this);
            let xValue = parseFloat(x.invert(numAry[0])).toFixed(2);
            let yValue = parseFloat(y.invert(numAry[1])).toFixed(2);
            let tooltipHtml = '';
            tooltipHtml += '<div style="margin-left: 15px;">测试哇</div>';
            tooltipHtml += '<div style="margin-left: 15px;">X:'+xValue+'</div>';
            tooltipHtml += '<div style="margin-left: 15px;">Y:'+yValue+'</div>';
            tooltip.html(tooltipHtml)
                .style("left",(d3.event.pageX) +"px")
                .style("top",(d3.event.pageY +20)+"px")
                .style("opacity",1.0)
        }).on("mousemove",function (d,i) {
            let numAry = d3.mouse(this);
            let xValue = parseFloat(x.invert(numAry[0])).toFixed(2);
            let yValue = parseFloat(y.invert(numAry[1])).toFixed(2);
            let tooltipHtml = '';
            tooltipHtml += '<div style="margin-left: 15px;">测试哇</div>';
            tooltipHtml += '<div style="margin-left: 15px;">X:'+xValue+'</div>';
            tooltipHtml += '<div style="margin-left: 15px;">Y:'+yValue+'</div>';
            tooltip.html(tooltipHtml)
                .style("left",(d3.event.pageX) +"px")
                .style("top",(d3.event.pageY +20)+"px")
                .style("opacity",1.0)
        }).on("mouseout",function (d,i) {
            tooltip.style("opacity",0.0);
        });

        svg.call(zoom);

        function brushed() {
            if (d3.event.sourceEvent && d3.event.sourceEvent.type === "zoom") return; // ignore brush-by-zoom
            var s = d3.event.selection || x2.range();
            x.domain(s.map(x2.invert, x2));
            focus.select(".area").attr("d", area);
            focus.select(".axis--x").call(xAxis);
            svg.select(".zoom").call(zoom.transform, d3.zoomIdentity
                .scale(width / (s[1] - s[0]))
                .translate(-s[0], 0));
        }

        function zoomed() {
            var t = d3.event.transform;
            x.domain(t.rescaleX(x2).domain());
            focus.select(".area").attr("d", area);
            focus.select(".axis--x").call(xAxis);
        }

        function updateLineChart() {

            this.updateD3ChartDate = function (updateData) {
                x.domain(d3.extent(updateData, function(d) {
                    console.log('d.date:',d.date);
                    return d.date;
                }));
                x2.domain(x.domain());
                xAxis = d3.axisBottom(x).ticks(30);

                area = d3.area()
                    .curve(d3.curveMonotoneX)
                    .x(function(d) { return x(d.date); })
                    .y0(height)
                    .y1(function(d) { return y(d.price); });

                focus.select("path")
                    .datum(updateData)
                    .attr("class", "area")
                    .attr("d", area);
                focus.select("g")
                    .attr("class", "axis axis--x")
                    .attr("transform", "translate(0," + height + ")")
                    .call(xAxis);

                function updateZoomed() {
                    var t = d3.event.transform;
                    x.domain(t.rescaleX(x2).domain());
                    focus.select(".area").attr("d", area);
                    focus.select(".axis--x").call(xAxis);
                }
            }
        }
        return new updateLineChart();
    },
    getD3Bar: function(){
        let height = 600
        let width = 750
        //画布周边的空白
        var padding = {left:30, right:30, top:20, bottom:20};
        var margin = {top: 20, right: 20, bottom: 110, left: 40};

        let dataset = [10,20,30,40,50,60,70,80,90,100]

        dataset = [];

        let xDomain = [];

        let count = 1000;

        for (let i = 1; i <= count; i++) {
            xDomain.push(i);
            dataset.push(Math.random() * 200)
        }

        let xScale = d3.scaleLinear().range([0, width]).domain(d3.extent(dataset, (d)=>{return d}));
        // x轴
        let xAxis = d3.axisBottom(xScale)


        //scaleExtent   Infinity
        var zoom = d3.zoom()
            .scaleExtent([1, 20])
            .translateExtent([[0, 0], [width, height]])
            .extent([[0, 0], [width,height]])
            .on("zoom", zoomed);

        let svg = d3.select('#d3Div').select('svg')
            .attr('width',width)
            .attr('height',height)
            .attr("class", "focus")
            .attr('class', 'mg50');

        // 柱状图
        let view = svg.append('g')
            .attr("width", width)
            .attr("height", height)
            .attr('transform', "translate(0, 250)");

        let rect = view.selectAll('rect').data(dataset).enter().append('rect')
            .attr('width',(d,i)=>{
                return 0.1;
            })
            .attr('height', (d)=>{
                console.log(d);
                return d
            })
            .attr('x', (d,i)=>{
                let x1 = width / dataset.length * i;
                return x1;
            })
            .attr('y', (d,i)=>{return -d})
            .attr('fill', 'blue');

        let gx = svg.append('g')
            .attr("class", "axis axis--x")
            .attr('transform', "translate(0, 250)")
            //.attr('class', 'x axis')
            .call(xAxis);

        svg.call(zoom);

        function zoomed() {// translate
            //rect.attr("transform", d3.event.transform);
            rect.attr('transform', 'translate('+d3.event.transform.x+',0)scale('+d3.event.transform.k+')');

            //view.attr("transform", d3.event.transform);
            gx.call(xAxis.scale(d3.event.transform.rescaleX(xScale)));
            //svg.select(".x.axis").call(xAxis);
        }
    },
    getD3: function(){
        debugger
        const chart = eventDrops({ d3 });

        const repositoriesData = repositories.map(repository => ({
            name: repository.name,
            data: repository.commits,
        }));

        repositoriesData.forEach(item=>{
            repositoriesData.data = repositoriesData.fullData;
        })

        console.log(repositoriesData);

        d3.select('#eventdrops-demo')
            .data([repositoriesData])
            .call(chart);
    },
    /*刷新echarts 渲染数据*/
    refEchartsData: function(){
        console.log(this);
        var dataCount = 5000;
        var data = this.generateData(dataCount);
        var option = {
            series: [
                {
                    name:'测试1',
                    type:'bar',
                    data:data.valueData,
                    large:true,
                    largeThreshold:500,
                    itemStyle:{
                        opacity:0
                    }
                },
            ]
        };
        this.myChart.setOption(option);
    },
    /*初始化echarts bar*/
    echartsBar: function(){
        var dataCount = 50000;
        var data = this.generateData(dataCount);

        this.myChart = this.$echarts.init(document.getElementById('echartsId'));

        var option = {
            grid: {
                right: '20%'
            },
            xAxis: {
                show:false
            },
            yAxis: {
                show:false
            },
            dataZoom: [{
                start: 0,
                end: 30,
                handleIcon: 'M10.7,11.9v-1.3H9.3v1.3c-4.9,0.3-8.8,4.4-8.8,9.4c0,5,3.9,9.1,8.8,9.4v1.3h1.3v-1.3c4.9-0.3,8.8-4.4,8.8-9.4C19.5,16.3,15.6,12.2,10.7,11.9z M13.3,24.4H6.7V23h6.6V24.4z M13.3,19.6H6.7v-1.4h6.6V19.6z',
                handleSize: '60%',
                handleStyle: {
                    color: '#fff',
                    shadowBlur: 2,
                    shadowColor: 'rgba(0, 0, 0, 1)',
                    shadowOffsetX: 2,
                    shadowOffsetY: 2
                },
                top:'top',
                left:50,
                height:100,
                showDetail:false,
            }],
            series: [
                {
                    name:'测试1',
                    type:'bar',
                    data:data.valueData,
                    large:true,
                    largeThreshold:500,
                    itemStyle:{
                        opacity:0
                    }
                },
            ]
        };
        this.myChart.setOption(option);
    },
    generateData: function (count) {
        var baseValue = Math.random() * 1000;
        var time = +new Date(2011, 0, 1);
        var smallBaseValue;

        function next(idx) {
            smallBaseValue = idx % 30 === 0
                ? Math.random() * 900
                : (smallBaseValue + Math.random() * 1500 - 250);
            baseValue += Math.random() * 20 - 10;
            return Math.max(
                0,
                Math.round(baseValue + smallBaseValue) + 3000
            );
        }

        var categoryData = [];
        var valueData = [];

        for (var i = 0; i < count; i++) {
            categoryData.push(i + "ms");
            valueData.push(next(i).toFixed(2));
            time += 1000;
        }

        return {
            categoryData: categoryData,
            valueData: valueData
        };
    }
}