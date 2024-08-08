import { FormEvent, useState } from 'react';
import DatePicker from 'react-datepicker';
import 'react-datepicker/dist/react-datepicker.css'; import { format } from 'date-fns'; import TaskStatusTable from './Table';
export const URL = "http://localhost:9999/"
const TimeAndDayPicker = () => {
  const [selectedDate, setSelectedDate] = useState(new Date());
  const [file, setFile] = useState<File | null>(null);

  const [retry, setRetry] = useState<number | undefined>();
  const [isLoading, setIsLoading] = useState(false);
  const [change, setChange] = useState(0);
  const handleDateChange = (date: Date | null) => {
    if (date) {
      setSelectedDate(date);
    }
  };

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsLoading(true);
    if (selectedDate && file) {

      const formattedDate = format(selectedDate, "yyyy-MM-dd HH:mm:ss");
      const utcDate = new Date(formattedDate).toISOString();

      //create task entry
      let created_task_body = {
        schedule_at: utcDate,
        retry: retry
      }
      try {
        let task_info = await fetch(URL + "task/create", {
          body: JSON.stringify(created_task_body),
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          }
        }).then(async res => {
          let json = await res.json();
          if (res.status == 200) {
            return json as { id: number, tracing_id: string };
          }

          throw new Error(JSON.stringify(json));

        });
        let s3body = {
          id: task_info.id,
          executable_size: file.size
        }
        let s3 = await fetch(URL + "signurl/create", {
          body: JSON.stringify(s3body),
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          }
        }).then(async res => {

          let json = await res.json();
          if (res.status == 200) {
            return json as { presigned_url: string };
          }

          throw new Error(JSON.stringify(json));
        })
        await fetch(s3.presigned_url, {
          method: "PUT",
          body: file,
          headers: {
            "Content-Type": "application/x-executable",
          }
        }).then(async res => {

          if (res.status != 200) {
            let text = await res.text();
            console.log(text);
            throw new Error("s3 error");
          }

        });
        let status = await fetch(URL + "file/status", {
          method: "POST",
          body: JSON.stringify({ id: task_info.id }),
          headers: {
            "Content-Type": "application/json",
          }
        }).then(async res => {
          let json = await res.json() as { status: string };
          if (res.status == 200) {
            return json;
          }
          throw new Error(JSON.stringify(json));
        })
        if (status.status == "PENDING") {
          throw new Error("File is not uploaded yet");
        }
        (document.getElementById('create-task-form') as HTMLFormElement)?.reset();
        setChange(change => change + 1);
        alert("Successfully submitted");

      } catch (err) {
        console.log("error", err);
        alert("Server Errorr");
      }

    }
    setIsLoading(false);
  };

  const weekend = (date: Date) => {
    const today = new Date();
    return date > today;
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      const selectedFile = e.target.files[0];
      if (selectedFile.type === "") {
        setFile(selectedFile);
      } else {
        alert("Can't support this file type. Please select an executable without an extension.");
        e.target.value = '';
      }
    }
  };

  return (
    <div >
      <form onSubmit={handleSubmit} id={"create-task-form"} className="max-w-md my-3 mx-auto mt-10 p-6 bg-white rounded-lg shadow-md">
        <div className="mb-4">
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="datePicker">
            Select Date and Time
          </label>
          <DatePicker
            id="datePicker"
            filterDate={weekend}
            selected={selectedDate}
            onChange={handleDateChange}
            showTimeSelect
            timeFormat="HH:mm:ss"
            timeIntervals={1}
            timeCaption="Time"
            dateFormat="MMMM d, yyyy h:mm:ss aa"
            className="w-full p-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div className="mb-4">
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="fileInput">
            Select Executable File
          </label>
          <input
            id="fileInput"
            type="file"
            onChange={handleFileChange}
            className="w-full p-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div className="mb-4">
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="fileInput">
            Select Total retry
          </label>
          <input
            id="fileInput"
            type="number"
            required={true}
            onChange={(e) => {
              let r = parseInt(e.target.value);
              if (r > 0 && r <= 3) {
                setRetry(r);
              } else {
                alert("Retry Value should be inbetween 0-3")
                e.target.value = "0";
              }
            }}
            className="w-full p-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        {!isLoading ?
          <button
            type="submit"
            className="w-full bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
          >
            Create Task
          </button> : <Spinner />}
      </form>
      <TaskStatusTable change={change} />
    </div>
  );
};
export const Spinner = () => (
  <div className="flex justify-center items-center">
    <svg className="animate-spin h-5 w-5 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
  </div>
);
export default TimeAndDayPicker;
