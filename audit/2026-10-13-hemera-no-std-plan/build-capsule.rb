require 'json';require 'digest';require 'fileutils';require 'tmpdir';require 'open3'
root='/tmp/nox-hemera-no-std-plan-v1';audit='/Users/master/cyber/nox-no-std-dependencies/audit/2026-10-13-hemera-no-std-plan'
raise 'frozen archive exists' if File.exist?(audit+'/evidence.tar.gz')
FileUtils.mkdir_p(audit)
files=Dir.glob(root+'/**/*',File::FNM_DOTMATCH).select{|p|File.file?(p)}.sort.to_h{|p|['plan/'+p.delete_prefix(root+'/'),p]}
files['verification/author.json']='/tmp/nox-hemera-no-std-plan-v1-verification.json'
files['verification/root.json']='/tmp/nox-hemera-no-std-root-plan-verification-v1.json'
index=[]
Dir.mktmpdir('nox-feature-capsule-') do |dest|
 FileUtils.mkdir_p(dest+'/objects')
 files.each do |name,path|
  b=File.binread(path);h=Digest::SHA256.hexdigest(b);out=dest+'/objects/'+h;File.binwrite(out,b) unless File.exist?(out)
  index << {path:name,bytes:b.bytesize,sha256:h}
 end
 list=dest+'/members.txt';File.write(list,index.map{|r|'objects/'+r[:sha256]}.uniq.sort.join("\n")+"\n")
 _,err,s=Open3.capture3('tar','-czf',audit+'/evidence.tar.gz','-C',dest,'-T',list);raise err unless s.success?
end
File.write(audit+'/index.json',JSON.pretty_generate(index)+"\n")
m=JSON.parse(File.read(root+'/manifest.json'));meta=m.merge('archive_sha256'=>Digest::SHA256.file(audit+'/evidence.tar.gz').hexdigest,'index_sha256'=>Digest::SHA256.file(audit+'/index.json').hexdigest,'logical_payloads'=>index.length,'unique_objects'=>index.map{|r|r[:sha256]}.uniq.length,'state_at_freeze'=>'Fresh Opus PLAN pending; production unchanged; root independently replayed this input.')
File.write(audit+'/capsule.json',JSON.pretty_generate(meta)+"\n")
FileUtils.cp(__FILE__,audit+'/build-capsule.rb')
puts JSON.pretty_generate(meta)
