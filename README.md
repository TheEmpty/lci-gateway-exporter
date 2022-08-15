# LCI Gateway Exporter
Currently only exports Generator, HVAC, and Tanks.
Makes a great companion to something like [Prometheus](https://prometheus.io/) and [Grafana.](https://grafana.com/)
Allowing you to create alarms for things HVAC temprature, tank level, or genrator run time.
This must be run on the same network as your LCI gateway. Eg, you should be able to see a JSON response [from the gateway.](http://192.168.1.4:8080/rest/things/)

## Docker Image
Quick start: `docker run -p 8888:8888 theempty/lci-gateway-exporter:latest`

Currently no configurable settings.

## Example output
```
# HELP lci_gateway_living_room_climate_zone_outside_temprature HVAC outside temprature
# TYPE lci_gateway_living_room_climate_zone_outside_temprature gauge
lci_gateway_living_room_climate_zone_outside_temprature 47.9375
# HELP lci_gateway_living_room_climate_zone_inside_temprature HVAC inside temprature
# TYPE lci_gateway_living_room_climate_zone_inside_temprature gauge
lci_gateway_living_room_climate_zone_inside_temprature 77
# HELP lci_gateway_living_room_climate_zone_fan Fan state. Auto = 0, Low = 1, High = 2
# TYPE lci_gateway_living_room_climate_zone_fan gauge
lci_gateway_living_room_climate_zone_fan 0
# HELP lci_gateway_living_room_climate_zone_mode A/C mode. Off = 0, Heat = 1, Cool = 2, Heat-Cool = 3
# TYPE lci_gateway_living_room_climate_zone_mode gauge
lci_gateway_living_room_climate_zone_mode 2

# HELP lci_gateway_garage_climate_zone_outside_temprature HVAC outside temprature
# TYPE lci_gateway_garage_climate_zone_outside_temprature gauge
lci_gateway_garage_climate_zone_outside_temprature 47.9375
# HELP lci_gateway_garage_climate_zone_inside_temprature HVAC inside temprature
# TYPE lci_gateway_garage_climate_zone_inside_temprature gauge
lci_gateway_garage_climate_zone_inside_temprature 75
# HELP lci_gateway_garage_climate_zone_fan Fan state. Auto = 0, Low = 1, High = 2
# TYPE lci_gateway_garage_climate_zone_fan gauge
lci_gateway_garage_climate_zone_fan 0
# HELP lci_gateway_garage_climate_zone_mode A/C mode. Off = 0, Heat = 1, Cool = 2, Heat-Cool = 3
# TYPE lci_gateway_garage_climate_zone_mode gauge
lci_gateway_garage_climate_zone_mode 2

# HELP lci_gateway_generator_state Generator state. Off = 0, Priming = 1, Starting = 2, Running = 3
# TYPE lci_gateway_generator_state gauge
lci_gateway_generator_state 0

# HELP lci_gateway_auxilliary_fuel_tank Tank percentage
# TYPE lci_gateway_auxilliary_fuel_tank gauge
lci_gateway_auxilliary_fuel_tank 0

# HELP lci_gateway_black_tank_2 Tank percentage
# TYPE lci_gateway_black_tank_2 gauge
lci_gateway_black_tank_2 33

# HELP lci_gateway_fresh_tank Tank percentage
# TYPE lci_gateway_fresh_tank gauge
lci_gateway_fresh_tank 0

# HELP lci_gateway_black_tank Tank percentage
# TYPE lci_gateway_black_tank gauge
lci_gateway_black_tank 33

# HELP lci_gateway_bedroom_climate_zone_outside_temprature HVAC outside temprature
# TYPE lci_gateway_bedroom_climate_zone_outside_temprature gauge
lci_gateway_bedroom_climate_zone_outside_temprature 47.9375
# HELP lci_gateway_bedroom_climate_zone_inside_temprature HVAC inside temprature
# TYPE lci_gateway_bedroom_climate_zone_inside_temprature gauge
lci_gateway_bedroom_climate_zone_inside_temprature 82
# HELP lci_gateway_bedroom_climate_zone_fan Fan state. Auto = 0, Low = 1, High = 2
# TYPE lci_gateway_bedroom_climate_zone_fan gauge
lci_gateway_bedroom_climate_zone_fan 0
# HELP lci_gateway_bedroom_climate_zone_mode A/C mode. Off = 0, Heat = 1, Cool = 2, Heat-Cool = 3
# TYPE lci_gateway_bedroom_climate_zone_mode gauge
lci_gateway_bedroom_climate_zone_mode 0

# HELP lci_gateway_grey_tank Tank percentage
# TYPE lci_gateway_grey_tank gauge
lci_gateway_grey_tank 33

# HELP lci_gateway_generator_fuel_tank Tank percentage
# TYPE lci_gateway_generator_fuel_tank gauge
lci_gateway_generator_fuel_tank 0
```

### TODO
* HVAC State
* Switches (ex: Water heater on for over X hours)

### Unplanned
* Dimmers
