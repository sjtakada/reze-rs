#!/usr/bin/ruby

require 'json'
require 'erb'

# global definition.
@rtype2code = {'Kernel' => 'K',
               'Static' => 'S',
               'Connected' => 'C',
               'Eigrp' => 'D',
               'Ospf' => 'O',
               'Isis' => 'i',
               'Rip' => 'R',
               'Bgp' => 'B'}

template_file = ARGV[0]
json_str = STDIN.read

@json = JSON.parse(json_str)

template = ERB.new(File.read(template_file), nil, '-')
puts template.result

