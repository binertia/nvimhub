package main

import (
	"fmt"
	"io"
	"log"
	"net/http"
)

//-------------------------------

func main() {
	// route
	http.HandleFunc("/", defaultHandler)
	http.HandleFunc("/data", fetchHandler)

	// port 8080 fetch on 8080/data
	log.Println("server on :8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		log.Fatal(err)
	}
}

func defaultHandler(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "we will fetch data from github.")
}

func fetchHandler(w http.ResponseWriter, r *http.Request) {
	resp, err := http.Get("http://localhost:8081/data")
	if err != nil {
		http.Error(w, "failed to fetch data from service", http.StatusInternalServerError)
		return
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		http.Error(w, "failed to read body", http.StatusInternalServerError)
		return
	}

	w.Header().Set("content", "application/json")
	w.Write(body)
}
