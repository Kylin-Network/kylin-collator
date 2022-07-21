
#include "./RSErasureCode.h"


int main(){
	// flt_roundtrip();

	init();//fill log table and exp table
	init_dec();//compute factors used in erasure decoder
	return roundtrip(32, 4);//test(n, k), k: message size, n: domain size
}
