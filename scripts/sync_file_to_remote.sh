#!/bin/bash
SOURCE_DIR="$HOME/Documents/task-scheduler/infra/"
DESTINATION="kube:~/infra"
rsync -av --delete  $SOURCE_DIR $DESTINATION
inotifywait -m -r -e modify,create,delete,move "$SOURCE_DIR" | while read path action file; do
  rsync -av --delete  $SOURCE_DIR $DESTINATION
done

rsync -avz "kube:~/infra" kube:"$HOME/Documents/task-scheduler/infra/"
