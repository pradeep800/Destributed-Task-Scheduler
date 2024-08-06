import { useEffect, useState } from 'react';
import relativeTime from "dayjs/plugin/relativeTime";
import dayjs from "dayjs";
import { Spinner } from './App';
interface Task {
  id: number;
  schedule_at: Date;
  status: string;
  total_retry: number;
  current_retry: number;
  tracing_id: string;
}
dayjs.extend(relativeTime);


const TaskStatusTable = ({ change }: { change: number }) => {
  let [tasks, setTasks] = useState<Task[]>([]);
  let [isLoading, setIsLoading] = useState(false);
  useEffect(() => {
    fetch("http://localhost:3000/task/all/status").then(async res => {
      let data = await res.json() as Task[];
      setTasks(data);
    })

  }, [change]);
  if (isLoading) {
    return <Spinner />;
  }
  if (tasks.length == 0) {
    return null;
  }
  return (
    <div className="overflow-x-auto m-3 ">
      <div className="flex  mb-2">
        <button
          className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-4 px-3 rounded text-xs"
          onClick={() => {
            setIsLoading(true)
            fetch("http://localhost:3000/task/all/status").then(async res => {
              let data = await res.json() as Task[];
              setTasks(data);
            }).catch(err => {
              console.log(err);
            }).finally(() => {
              setIsLoading(false);
            })

          }}
        >
          Refresh
        </button>
      </div>
      <table className="min-w-full bg-white shadow-md rounded-lg overflow-hidden">
        <thead className="bg-gray-100">
          <tr>
            <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">ID</th>
            <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Schedule At</th>
            <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
            <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Total Retry</th>
            <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Current Retry</th>
            <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Tracing ID</th>
            <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>

          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200">
          {tasks.map((task) => (
            <tr key={task.id}>
              <td className="px-4 py-2 whitespace-nowrap">{task.id}</td>
              <td className="px-4 py-2 whitespace-nowrap">{dayjs(task.schedule_at).fromNow()}</td>
              <td className="px-4 py-2 whitespace-nowrap">{task.status}</td>
              <td className="px-4 py-2 whitespace-nowrap">{task.total_retry}</td>
              <td className="px-4 py-2 whitespace-nowrap">{task.current_retry}</td>
              <td className="px-4 py-2 whitespace-nowrap">{task.tracing_id}</td>
              <td className="px-4 py-2 whitespace-nowrap">
                <button
                  onClick={() => {
                    //we know that we can't delete anything so we can use index

                    fetch("http://localhost:3000/task/status", {
                      method: "POST",
                      body: JSON.stringify({ id: task.id }),
                      headers: {
                        "Content-Type": "application/json",
                      }
                    }).then(async res => {
                      let json = await res.json();
                      if (res.status == 400) {
                        alert("Please upload file");
                        return;
                      }
                      setTasks(tasks => {
                        tasks[task.id - 1] = json;
                        return tasks;
                      });
                    })
                  }}
                  className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-2 rounded text-xs"
                >
                  Refresh
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default TaskStatusTable;
