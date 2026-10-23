#!/bin/bash
# tart-par1 tie-break (parity-method.md): MOD-11 (group T09) on Tart with its UTM baseline candidate df3d8c5d1c.
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-par1-20261020
cd $Q
until grep -q "STOP file present" runs/seq.log || [ "$(grep -c ALL-DONE runs/seq.log)" -ge 2 ]; do sleep 30; done
mv STOP logs/STOP.used-$(date -u +%Y%m%dT%H%M%SZ)
echo "$(date -u +%FT%TZ) tie-break start: T09 on baseline candidate df3d8c5d1c (replaces the T21 replay under the cap)" >> runs/seq.log
CAND=$Q/candidate-tb-df3d8c5d1c ./h/make_stage.sh $Q/packet-v4p-tb-df3d8c5d1c > logs/stage-tiebreak.out 2>&1 || { echo "$(date -u +%FT%TZ) tie-break stage failed" >> runs/seq.log; exit 1; }
cp h/expected.json logs/expected-tiebreak.json
PAR_LABEL=tiebreak-df3d8c5d1c ./run-groups.sh T09 > logs/tiebreak.out 2>&1
./h/make_stage.sh $Q/packet-v4p > logs/stage-restore.out 2>&1
echo "$(date -u +%FT%TZ) stage restored to the run candidate: $(cat h/expected.json)" >> runs/seq.log
