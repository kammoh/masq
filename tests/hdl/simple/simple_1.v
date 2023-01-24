module simple_1(clk, rst, in_data, in_valid, in_ready, out_data, out_valid, out_ready);
    localparam W = 2;
    input clk;
    input rst;
    input [W-1:0] in_data;
    input in_valid;
    output in_ready;
    output [W-1:0] out_data;
    output out_valid;
    input out_ready;

    logic [W-1:0] x;
    logic v;

    assign x[W-1:W/2] = in_data[W-1:W/2] ^ in_data[W/2-1:0];
    assign x[W/2-1:0] = in_data[W-1:W/2] & in_data[W/2-1:0];
    
    assign in_ready = out_ready || !v;
    assign out_valid = v;

    always_ff @( posedge clk ) begin
        if (in_valid && in_ready) begin
            out_data <= x;
            v <= 1;
        end
        else if (out_valid && out_ready) begin
            v <= 0;
        end
        if (rst)
            v <= 0;
    end

endmodule