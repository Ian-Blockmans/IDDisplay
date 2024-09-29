use pyo3::prelude::*;

//struct Song{
//    track_name: String,
//    artist_name: String,
//    art: String,
//}
//enum Message {
//    Detect,
//    Exit,
//}
//
//impl Song {
//    fn update(&mut self, message: Message) {
//        match message {
//            Message::Detect => {}
//            Message::Exit => {}
//        }
//    }
//}

//#[pyo3_asyncio::async_std::main]
//async fn main() -> PyResult<()> {
//    let fut = Python::with_gil(|py| {
//        let asyncio = py.import("asyncio")?;
//        // convert asyncio.sleep into a Rust Future
//        pyo3_asyncio::async_std::into_future(asyncio.call_method1("sleep", (1.into_py(py),))?)
//    })?;
//
//    fut.await?;
//
//    Ok(())
//}

fn shazamrec() -> Result<String, PyErr> {
    let shazamio_py_file = include_str!("../python/ShazamIO.py");

    Python::with_gil(|py| {
        let shazam= PyModule::from_code_bound(py, shazamio_py_file, "ShazamIO.py", "shazam")?.getattr("shazam_rec")?;
        let sound_file = ("MPH-CrowdRolling.ogg",);
        let song_rec_return = shazam.call1( sound_file)?.s;
        Ok(song_rec_return.extract()?)
    })
    //let shazam_return: Bound<'py, PyString> = PyString::new_bound(py, "init"); return from python

}

fn main() -> Result<(), PyErr> {
    pyo3::prepare_freethreaded_python();
    let _shazam_json_s = shazamrec()?; //TODO: map error to main return result error 
    Ok(())
    //let _shazam_json_p = json::parse(&shazam_json_s);
    //shazam_json_p[]
    
    //let shazam_return: Bound<'py, PyString> = PyString::new_bound(py, "init"); return from python

}