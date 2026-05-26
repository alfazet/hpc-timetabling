#!/bin/bash

#SBATCH -A stud-2526-l-02
#SBATCH -p student
#SBATCH -G 1
#SBATCH -J timetabling
#SBATCH -o %x-%j.out

./main "$@"
