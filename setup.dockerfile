# syntax = docker/dockerfile:1.2
FROM scratch as chef  
WORKDIR /app
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /app/stark_verify /stark/stark_verify
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /app/stark_verify.dat /stark/stark_verify.dat
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /app/stark_verify_final.zkey /stark/stark_verify_final.zkey
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /usr/local/sbin/rapidsnark /stark/rapidsnark
