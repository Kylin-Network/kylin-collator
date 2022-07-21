/*
Encoding/erasure decoding for Reed-Solomon codes over binary extension fields
Author: Sian-Jheng Lin (King Abdullah University of Science and Technology (KAUST), email: sianjheng.lin@kaust.edu.sa)

This program is the implementation of
Lin, Han and Chung, "Novel Polynomial Basis and Its Application to Reed-Solomon Erasure Codes," FOCS14.
(http://arxiv.org/abs/1404.3458)
*/

#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdint.h>
#include <stdio.h>


#include "RSErasureCode.h"

/*
typedef unsigned char GFSymbol;
#define FIELD_BITS 8//2^FIELD_BITS: the size of Galois field
GFSymbol mask = 0x1D; //GF(2^8): x^8 + x^4 + x^3 + x^2 + 1
GFSymbol Base[] = {1, 214, 152, 146, 86, 200, 88, 230};//Cantor basis
*/

typedef unsigned short GFSymbol;
#define FIELD_BITS 16
GFSymbol mask = 0x2D;//x^16 + x^5 + x^3 + x^2 + 1
GFSymbol Base[FIELD_BITS] = {1, 44234, 15374, 5694, 50562, 60718, 37196, 16402, 27800, 4312, 27250, 47360, 64952, 64308, 65336, 39198};//Cantor basis

#define FIELD_SIZE (1<<FIELD_BITS)//Field size
#define MODULO (FIELD_SIZE-1)

GFSymbol LOG_TABLE[FIELD_SIZE];
GFSymbol EXP_TABLE[FIELD_SIZE];

//-----Used in decoding procedure-------
GFSymbol skewVec[MODULO];//twisted factors used in FFT
GFSymbol B[FIELD_SIZE>>1];//factors used in formal derivative
GFSymbol log_walsh[FIELD_SIZE];//factors used in the evaluation of the error locator polynomial

//return a*EXP_TABLE[b] over GF(2^r)
GFSymbol mulE(GFSymbol a, GFSymbol b){
	return a ? EXP_TABLE[(LOG_TABLE[a]+b &MODULO) + (LOG_TABLE[a]+b >>FIELD_BITS)]: 0;
}

void walsh(GFSymbol* data, int size){//fast Walsh–Hadamard transform over MODULOulo MODULO
	for (int depart_no=1; depart_no<size; depart_no <<= 1){
		for (int j = 0; j < size; j += depart_no<<1){
			for (int i=j; i<depart_no+j; i++){
				unsigned tmp2 = data[i] + MODULO - data[i+depart_no];
				data[i] = (data[i] + data[i+depart_no]&MODULO) + (data[i] + data[i+depart_no]>>FIELD_BITS);
				data[i+depart_no] = (tmp2&MODULO) + (tmp2>>FIELD_BITS);
			}
		}
	}
	return;
}

void formal_derivative(GFSymbol* cos, int size){//formal derivative of polynomial in the new basis
	for(int i=1; i<size; i++){
		int leng = ((i^i-1)+1)>>1;
		for(int j=i-leng; j<i; j++)
			cos[j] ^= cos[j+leng];
	}
	// FIXME was `Size` before, so should be `FIELD_SIZE` now
	// FIXME but that would require `cos` to be of size
	// FIXME `FIELD_SIZE` too, so technically this could just be dropped imho
	// for(int i=size; i<FIELD_SIZE; i<<=1)
	// 	for(int j=0; j<size; j++)
	// 		cos[j] ^= cos[j+i];
	return;
}

void IFLT(GFSymbol* data, int size, int index){//IFFT in the proposed basis
	for (int depart_no=1; depart_no<size; depart_no <<= 1){
		for (int j=depart_no; j < size; j += depart_no<<1){
			for (int i=j-depart_no; i<j; i++)
				data[i+depart_no] ^= data[i];

			GFSymbol skew = skewVec[j+index-1];
			if (skew != MODULO)
				for (int i=j-depart_no; i<j; i++)
					data[i] ^= mulE(data[i+depart_no], skew);
		}
	}
	return;
}

//FFT in the proposed basis
void FLT(GFSymbol* data, int size, int index){
	for(int depart_no = size>>1; depart_no > 0; depart_no >>= 1){
		for (int j = depart_no; j < size; j += depart_no<<1){
			GFSymbol skew = skewVec[j+index-1];
			if (skew != MODULO)
				for (int i=j-depart_no; i<j; i++)
					data[i] ^= mulE(data[i+depart_no], skew);
			for (int i=j-depart_no; i<j; i++)
				data[i+depart_no] ^= data[i];
		}
	}
	return;
}

//initialize LOG_TABLE[], EXP_TABLE[]
void init(){
	GFSymbol mas = (1<<FIELD_BITS-1)-1;
	GFSymbol state=1;
	for(int i=0; i<MODULO; i++){
		EXP_TABLE[state]=i;
        if(state>>FIELD_BITS-1){
        	state &= mas;
        	state = state<<1^mask;
        }else
        	state <<= 1;
    }
    EXP_TABLE[0] = MODULO;

    LOG_TABLE[0] = 0;
	for(int i=0; i<FIELD_BITS; i++)
		for(int j=0; j<1<<i; j++)
			LOG_TABLE[j+(1<<i)] = LOG_TABLE[j] ^ Base[i];
    for(int i=0; i<FIELD_SIZE; i++)
        LOG_TABLE[i]=EXP_TABLE[LOG_TABLE[i]];

    for(int i=0; i<FIELD_SIZE; i++)
        EXP_TABLE[LOG_TABLE[i]]=i;
    EXP_TABLE[MODULO] = EXP_TABLE[0];
}


void init_dec(){//initialize skewVec[], B[], log_walsh[]
	GFSymbol base[FIELD_BITS-1];

	for(int i=1; i<FIELD_BITS; i++)
		base[i-1] = 1<<i;

	for(int m=0; m<FIELD_BITS-1; m++){
		int step = 1<<(m+1);
		skewVec[(1<<m)-1] = 0;
		for(int i=m; i<FIELD_BITS-1; i++){
			int s = 1<<(i+1);
			for(int j=(1<<m)-1; j<s; j+=step)
				skewVec[j+s] = skewVec[j] ^ base[i];
		}
		base[m] = MODULO-LOG_TABLE[mulE(base[m], LOG_TABLE[base[m]^1])];
		for(int i=m+1; i<FIELD_BITS-1; i++)
			base[i] = mulE(base[i], (LOG_TABLE[base[i]^1]+base[m])%MODULO);
	}
	for(int i=0; i<FIELD_SIZE; i++)
		skewVec[i] = LOG_TABLE[skewVec[i]];

	base[0] = MODULO-base[0];
	for(int i=1; i<FIELD_BITS-1; i++)
		base[i] = (MODULO-base[i]+base[i-1])%MODULO;

	B[0] = 0;
	for(int i=0; i<FIELD_BITS-1; i++){
		int depart = 1<<i;
		for(int j=0; j<depart; j++)
			B[j+depart] = (B[j] + base[i])%MODULO;
	}

	memcpy(log_walsh, LOG_TABLE, FIELD_SIZE * sizeof(GFSymbol));
	log_walsh[0] = 0;
	walsh(log_walsh, FIELD_SIZE);
}

void setup() {
	init();
	init_dec();
}

//Encoding alg for k/n<0.5: message is a power of two
void encodeL(GFSymbol* data, int k, GFSymbol* codeword, int n){
	memcpy(codeword, data, sizeof(GFSymbol)*k);
	IFLT(codeword, k, 0);
	for(int i=k; i<n; i+=k){
		memcpy(&codeword[i], codeword, sizeof(GFSymbol)*k);
		FLT(&codeword[i], k, i);
	}
	memcpy(codeword, data, sizeof(GFSymbol)*k);
}

// void encodeH(GFSymbol* data, int k, GFSymbol* parity, GFSymbol* mem, int n){
// //Encoding alg for k/n>0.5: parity is a power of two.
// //data: message array. parity: parity array. mem: buffer(size>= n-k)
// 	int t = n-k;
// 	memset(parity, 0, sizeof(GFSymbol)*t);
// 	for(int i=t; i<n; i+=t){
// 		memcpy(mem, &data[i-t], sizeof(GFSymbol)*t);
// 		IFLT(mem, t, i);
// 		for(int j=0; j<t; j++)
// 			parity[j] ^= mem[j];
// 	}
// 	FLT(parity, t, 0);
// }

//Compute the evaluations of the error locator polynomial
void decode_init(Boolean* erasure, GFSymbol* log_walsh2, int n){
	for(int i=0; i<n; i++)
		log_walsh2[i] = erasure[i];
	walsh(log_walsh2, FIELD_SIZE);
	for (int i=0; i<n; i++)
		log_walsh2[i] = (unsigned long)log_walsh2[i]*log_walsh[i]%MODULO;
	walsh(log_walsh2, FIELD_SIZE);
	for (int i=0; i<n; i++)
		if(erasure[i]) log_walsh2[i] = MODULO-log_walsh2[i];
}

void decode_main(GFSymbol* codeword, int k, Boolean* erasure, GFSymbol* log_walsh2, int n){
	int recover_chunks = k;//k2 can be replaced with k
	for (int i=0; i<n; i++)
		codeword[i] = erasure[i]
			?
			0
			:
			mulE(
				codeword[i],
				log_walsh2[i]
				);
	IFLT(codeword, n, 0);

	for(int i=0; i<n; i+=2) {//formal derivative
		codeword[i] = mulE(codeword[i], MODULO-B[i>>1]);
		codeword[i+1] = mulE(codeword[i+1], MODULO-B[i>>1]);
	}

	formal_derivative(codeword, n);
	for(int i=0; i<n; i+=2){
		codeword[i] = mulE(codeword[i], B[i>>1]);
		codeword[i+1] = mulE(codeword[i+1], B[i>>1]);
	}

	FLT(codeword, n, 0);

	for (int i=0; i<recover_chunks; i++) {
		codeword[i] = erasure[i]? mulE(codeword[i], log_walsh2[i]) : 0;
	}
}

void print_sha256(char* txt, uint8_t* data, size_t lx) {
#if 0
	uint8_t hash[32];
	memset(hash, 0x00, 32);
	calc_sha_256(hash, data, lx);
	printf("sha256(c|%s):\n", txt);
	for(int i=0; i<32; i++) {
		printf("%02x", hash[i]);
	}
	printf("\n");
#endif
}

int roundtrip(int n, int k) {
	//-----------Generating message----------
	GFSymbol data[n];
	const int nx2 = 2 * n;
	memset(data, 0x0, nx2);

	srand(time(NULL));
	// for(int i=n-k; i<n; i++)
    for(int i=0; i<k; i++) {
		data[i] = i*i % MODULO;
		// data[i] = rand()&MODULO;//filled with random numbers
	}


	printf("Message(Last n-k are zeros): \n");
	for(int i=0; i<k; i++) {
		printf("%04x ", data[i]);
	}
	printf("\n");

	print_sha256("data", (uint8_t*)data, nx2);

	//---------encoding----------
	GFSymbol codeword[n];
	memset(codeword, 0x00, n);

	encodeL(data, k, codeword, n);

	print_sha256("encoded", (uint8_t*)codeword, nx2);


	//--------erasure simulation---------
	Boolean erasure[FIELD_SIZE];
	memset(erasure, 0x00, FIELD_SIZE);

	for(int i=0; i<(n-k); i++)
		erasure[i] = 1;

	#if 0
		for(int i=n-1; i>0; i--){//permuting the erasure array
			int pos = rand()%(i+1);
			if(i != pos){
				Boolean tmp = erasure[i];
				erasure[i] = erasure[pos];
				erasure[pos] = tmp;
			}
		}
	#endif

	for (int i=0; i<n; i++) {
		if(erasure[i]) codeword[i] = 0;
	}


	print_sha256("erased", (uint8_t*)codeword, nx2);

	//---------Erasure decoding----------------
	GFSymbol log_walsh2[FIELD_SIZE];
	decode_init(erasure, log_walsh2, FIELD_SIZE);

	print_sha256("log_walsh2", (uint8_t*)log_walsh2, nx2);

	//---------main processing----------
	decode_main(codeword, k, erasure, log_walsh2, n);

	print_sha256("recovered", (uint8_t*)codeword, k * 2);

	printf("Decoded result:\n");
	for(int i=0; i<n; i++){
		if(erasure[i]) printf("%04X ", codeword[i]);
		else printf("%04X ", data[i]);
	}
	printf("\n");

	for (int i=0; i<k; i++){//Check the correctness of the result
		if(erasure[i] == 1) {
			if(data[i] != codeword[i]){
				printf("XXXX🐍XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\n");
				printf("XXXXXXXXXXXXX Decoding ERROR! XXXXXXX🐍XXXXXX\n");
				printf("XXXXXXXXXXXXXXXX🐍XXXXXXXXXXXXXXXXXXXXXXXXXXX\n");
				return 1;
			}
		}
	}
	printf(">>>>>>>>> 🎉🎉🎉🎉\n");
	printf(">>>>>>>>> > Decoding is **SUCCESS** ful! 🎈\n");
	printf(">>>>>>>>>\n");
	return 0;
}



#include <assert.h>

int test_flt_roundtrip() {
	const int N = 16;
	GFSymbol expected[16] = {
		1, 2, 3, 5, 8, 13, 21, 44,
		65, 0, 0xFFFF, 2, 3, 5, 7, 11,
	};
	GFSymbol data[N];
	memcpy(data, expected, N * sizeof(GFSymbol));


	FLT(data, N, N/4);
	printf("novel basis(c)\n");
	for(int i=0; i<N; i++){
		printf("%04X ", data[i]);
	}
	printf("\n");
	IFLT(data, N, N/4);
	for(int i=0; i<N; i++){
		assert(data[i] == expected[i]);
	}
	return 0;
}
