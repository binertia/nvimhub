# Use the official Go image from Docker Hub as the base image
FROM golang:1.23.4

# Set the working directory inside the container
WORKDIR /app

# Copy the go.mod and go.sum files first to leverage Docker caching
COPY go.mod ./

# Download Go dependencies (if any) to optimize Docker layer caching
RUN go mod tidy

# Copy the Go source code into the container
COPY . .

# Build the Go application
RUN go build -o api_gateway .

# Expose the port your Go API will run on
EXPOSE 8080

# Set the entrypoint to run the Go binary
CMD ["./api_gateway"]
