#!/usr/bin/bash

ELF=$1
ELF_DIR=$(dirname $ELF)

NAME=$(basename $ELF)
OLD=${ELF_DIR}/_old
DIFF=${ELF_DIR}/_diff

# backup old files to "_old" directory
mkdir -p ${OLD}
mv ${ELF}_src.s ${OLD}
mv $ELF.s ${OLD}
mv ${ELF}_sections.s ${OLD}
mv ${ELF}_readelf.txt ${OLD}
mv $ELF.isr_vector.txt ${OLD}
mv $ELF.text.txt ${OLD}
mv ${ELF}_size.txt ${OLD}
mv $ELF.bss.txt ${OLD}
mv $ELF.data.txt ${OLD}
mv $ELF.noinit.txt ${OLD}

arm-none-eabi-objdump -S $ELF | rustfilt > ${ELF}_src.s
arm-none-eabi-objdump -d $ELF | rustfilt > $ELF.s
arm-none-eabi-objdump -h $ELF | rustfilt > ${ELF}_sections.s
arm-none-eabi-readelf -a $ELF | rustfilt > ${ELF}_readelf.txt
arm-none-eabi-readelf -x .isr_vector $ELF > $ELF.isr_vector.txt
arm-none-eabi-readelf -x .text $ELF > $ELF.text.txt
arm-none-eabi-nm --print-size --size-sort --radix=x $ELF | rustfilt > ${ELF}_size.txt
arm-none-eabi-readelf -x .bss $ELF > $ELF.bss.txt
arm-none-eabi-readelf -x .data $ELF > $ELF.data.txt
arm-none-eabi-readelf -x .noinit $ELF > $ELF.noinit.txt

# calculate the diff between the old and new files
mkdir -p ${DIFF}
diff ${OLD}/${NAME}_src.s ${ELF}_src.s > ${DIFF}/${NAME}_src.diff
diff ${OLD}/$NAME.s $ELF.s > ${DIFF}/$ELF.diff
diff ${OLD}/${NAME}_sections.s ${ELF}_sections.s > ${DIFF}/${NAME}_sections.diff
diff ${OLD}/${NAME}_readelf.txt ${ELF}_readelf.txt > ${DIFF}/${NAME}_readelf.diff
diff ${OLD}/$NAME.isr_vector.txt $ELF.isr_vector.txt > ${DIFF}/$NAME.isr_vector.diff
diff ${OLD}/$NAME.text.txt $ELF.text.txt > ${DIFF}/$NAME.text.diff
diff ${OLD}/${NAME}_size.txt ${ELF}_size.txt > ${DIFF}/${NAME}_size.diff
diff ${OLD}/$NAME.bss.txt $ELF.bss.txt > ${DIFF}/$NAME.bss.diff
diff ${OLD}/$NAME.data.txt $ELF.data.txt > ${DIFF}/$NAME.data.diff
diff ${OLD}/$NAME.noinit.txt $ELF.noinit.txt > ${DIFF}/$NAME.noinit.diff

echo "workdir: ${ELF}"
