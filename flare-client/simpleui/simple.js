

function fill_ts_data(thread_ts_data, thread_start_time, thread_end_time, start_time, end_time, unit_time_ms) {
    let fill_steps_before = (thread_start_time - start_time)/unit_time_ms;
    let fill_steps_after = (end_time - thread_end_time)/unit_time_ms;
    if (fill_steps_before < 1 && fill_steps_after < 1) {
        return thread_ts_data;
    }

    let new_data_vec = [];// Vec::with_capacity(data_vec.len()+(fill_steps_before+fill_steps_after) as usize);
    for (var i=0; i<fill_steps_before; i++) {
        new_data_vec.push(0);
    }

    new_data_vec = new_data_vec.concat(thread_ts_data);

    for (var i=0; i<fill_steps_after; i++) {
        new_data_vec.push(0);
    }
    return new_data_vec;
}

//"hello {0}, {1}".format("Tom", "Great")
String.prototype.format = function()
{
    var args = arguments;
    return this.replace(/\{(\d+)\}/g,
        function(m,i){
            return args[i];
        });
}
