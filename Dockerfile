FROM golang:1.18-bullseye AS build

WORKDIR /build

COPY go.mod go.sum ./
RUN go mod download

COPY . ./
# # Unit tests
# RUN go test -v
RUN go build -ldflags="-w -s" -o /app .

##############
FROM gcr.io/distroless/static-debian11

COPY --from=build /app /app
CMD ["/app"]
